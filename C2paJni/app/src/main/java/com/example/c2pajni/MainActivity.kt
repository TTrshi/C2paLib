package com.example.c2pajni

import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : AppCompatActivity(), ResultCallback {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContentView(R.layout.activity_main)
        ViewCompat.setOnApplyWindowInsetsListener(findViewById(R.id.main)) { v, insets ->
            val systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
            v.setPadding(systemBars.left, systemBars.top, systemBars.right, systemBars.bottom)
            insets
        }

        // コールバックを登録
        NativeBridge.registerCallback(this)

        // Kotlin → Rust
        val res1 = NativeBridge.callRustFunction("First call from Kotlin")
        println(res1)  // => counter = 1

        // Rust → Kotlin（コールバック）→ Rust
        NativeBridge.triggerKotlinFromRust()
    }

    override fun onResult(msg: String) {
        println("Kotlin received from Rust: $msg")

        // コールバックの中で再度 Rust を呼び出す（同じ AppState にアクセス）
        val res = NativeBridge.callRustFunction("Back to Rust")
        println("Rust replied: $res")  // counter が増加
    }
}