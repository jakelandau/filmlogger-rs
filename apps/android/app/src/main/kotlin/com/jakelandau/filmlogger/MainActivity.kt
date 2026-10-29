package com.jakelandau.filmlogger

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.ui.graphics.Color
import com.jakelandau.filmlogger.ui.MainScreen

/** Burgundy accent, lightened for legibility as text on the dark theme. */
private val Burgundy = Color(0xFFBF6B7D)
/** Deep burgundy, for container fills paired with light content. */
private val BurgundyDeep = Color(0xFF800020)

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            // Always dark, regardless of the device's system setting, with a
            // burgundy accent in place of the default purple.
            MaterialTheme(
                colorScheme = darkColorScheme(
                    primary = Burgundy,
                    onPrimary = Color.White,
                    primaryContainer = BurgundyDeep,
                    onPrimaryContainer = Color.White,
                )
            ) {
                MainScreen()
            }
        }
    }
}
