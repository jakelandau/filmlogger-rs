package ca.jakelandau.filmlogger

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import coil.Coil
import coil.ImageLoader
import coil.decode.SvgDecoder
import ca.jakelandau.filmlogger.ui.MainScreen

/** Burgundy accent, lightened for legibility as text on the dark theme. */
private val Burgundy = Color(0xFFBF6B7D)
/** Deep burgundy, for container fills paired with light content. */
private val BurgundyDeep = Color(0xFF800020)

private val AppTypography = Typography(
    bodySmall = TextStyle(fontSize = 14.sp, lineHeight = 20.sp),
    bodyMedium = TextStyle(fontSize = 15.sp, lineHeight = 22.sp),
    bodyLarge = TextStyle(fontSize = 16.sp, lineHeight = 24.sp),
    labelMedium = TextStyle(fontSize = 13.sp, lineHeight = 18.sp),
    labelLarge = TextStyle(fontSize = 14.sp, lineHeight = 20.sp, fontWeight = FontWeight.Medium),
    titleSmall = TextStyle(fontSize = 16.sp, lineHeight = 22.sp, fontWeight = FontWeight.SemiBold),
    titleMedium = TextStyle(fontSize = 18.sp, lineHeight = 24.sp, fontWeight = FontWeight.SemiBold),
    titleLarge = TextStyle(fontSize = 22.sp, lineHeight = 28.sp, fontWeight = FontWeight.SemiBold),
    headlineSmall = TextStyle(fontSize = 26.sp, lineHeight = 32.sp, fontWeight = FontWeight.SemiBold),
)

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Coil.setImageLoader(
            ImageLoader.Builder(this)
                .components { add(SvgDecoder.Factory()) }
                .build()
        )
        setContent {
            // Always dark, regardless of the device's system setting, with a
            // burgundy accent in place of the default purple.
            MaterialTheme(
                colorScheme = darkColorScheme(
                    primary = Burgundy,
                    onPrimary = Color.White,
                    primaryContainer = BurgundyDeep,
                    onPrimaryContainer = Color.White,
                ),
                typography = AppTypography,
            ) {
                MainScreen()
            }
        }
    }
}
