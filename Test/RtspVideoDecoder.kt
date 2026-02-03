package com.example.rtsptest.rtsp

import android.media.Image
import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaFormat
import android.os.Build
import android.os.Handler
import android.os.HandlerThread
import android.view.Surface
import androidx.media3.common.util.SystemClock
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

class RtspVideoDecoder(
    private val surface: Surface,
) {

    private var mediaCodec: MediaCodec? = null
    private val bufferInfo = MediaCodec.BufferInfo()

    private val decodeThread = HandlerThread("VideoDecodeThread")
    private lateinit var handler: Handler

    private val mlExecutor = Executors.newSingleThreadExecutor()
    private val analyzing = AtomicBoolean(false)

    private var lastAnalyzeTime = 0L

    private val isSamsung =
        Build.MANUFACTURER.equals("samsung", true)
    private val isPixel =
        Build.MODEL.startsWith("Pixel")

    private val analyzeIntervalMs = when {
        isSamsung -> 300L
        isPixel -> 150L
        else -> 200L
    }

    fun start(format: MediaFormat) {
        decodeThread.start()
        handler = Handler(decodeThread.looper)

        handler.post {
            setupCodec(format)
            decodeLoop()
        }
    }

    fun stop() {
        handler.post {
            try {
                mediaCodec?.stop()
                mediaCodec?.release()
            } catch (_: Exception) {
            }
            decodeThread.quitSafely()
            mlExecutor.shutdown()
        }
    }

    // =========================
    // MediaCodec 初期化
    // =========================
    private fun setupCodec(format: MediaFormat) {
        format.setInteger(
            MediaFormat.KEY_COLOR_FORMAT,
            MediaCodecInfo.CodecCapabilities.COLOR_FormatYUV420Flexible
        )
        format.setInteger(MediaFormat.KEY_PRIORITY, 0)

        if (isSamsung) {
            format.setInteger(
                MediaFormat.KEY_OPERATING_RATE,
                Short.MAX_VALUE.toInt()
            )
        }

        val mime = format.getString(MediaFormat.KEY_MIME)!!
        mediaCodec = MediaCodec.createDecoderByType(mime)
        mediaCodec!!.configure(format, surface, null, 0)
        mediaCodec!!.start()
    }

    // =========================
    // RTSP から NAL 投入
    // =========================
    fun queueNal(data: ByteArray, ptsUs: Long) {
        handler.post {
            val codec = mediaCodec ?: return@post
            val index = codec.dequeueInputBuffer(10_000)
            if (index >= 0) {
                val buf = codec.getInputBuffer(index)!!
                buf.clear()
                buf.put(data)
                codec.queueInputBuffer(
                    index,
                    0,
                    data.size,
                    ptsUs,
                    0
                )
            }
        }
    }

    // =========================
    // デコードループ
    // =========================
    private fun decodeLoop() {
        val codec = mediaCodec ?: return

        while (true) {
            val index = codec.dequeueOutputBuffer(bufferInfo, 5_000)
            when {
                index >= 0 -> handleOutput(codec, index)
                index == MediaCodec.INFO_TRY_AGAIN_LATER -> Thread.yield()
                index == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED -> {}
            }
        }
    }

    // =========================
    // Surface + MLKit 分岐
    // =========================
    private fun handleOutput(codec: MediaCodec, index: Int) {
        val image = codec.getOutputImage(index)

        if (image != null && shouldAnalyze()) {
            dispatchToMlKit(image)
        } else {
            image?.close()
        }

        codec.releaseOutputBuffer(index, true)
    }

    private fun shouldAnalyze(): Boolean {
//        val now = SystemClock.elapsedRealtime()
//        if (now - lastAnalyzeTime > analyzeIntervalMs) {
//            lastAnalyzeTime = now
//            return true
//        }
        return false
    }

    private fun dispatchToMlKit(image: Image) {
        if (!analyzing.compareAndSet(false, true)) {
            image.close()
            return
        }

        mlExecutor.execute {
            try {
                //mlKitAnalyzer?.analyze(image)
            } finally {
                image.close()
                analyzing.set(false)
            }
        }
    }
}
