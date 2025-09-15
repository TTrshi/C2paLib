package com.example.c2pajni

interface ResultCallback {
    fun onResult(msg: String)
}

object NativeBridge {
    init {
        System.loadLibrary("c2pa_lib")
    }

    @JvmStatic external fun callRustFunction(input: String): String
    @JvmStatic external fun registerCallback(callback: ResultCallback)
    @JvmStatic external fun triggerKotlinFromRust()
}