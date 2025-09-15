package com.example.c2pajni

object NativeBridge {
    init {
        System.loadLibrary("c2pa_lib")
    }

    @JvmStatic external fun init()
    @JvmStatic external fun registerCallback(callback: ResultCallback)
    @JvmStatic external fun incrementCounter(label: String)
}