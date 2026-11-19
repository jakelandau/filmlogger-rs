package ca.jakelandau.filmlogger.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import coil.compose.AsyncImagePainter

// Official rating symbols bundled from Wikimedia Commons. Boards without an
// official symbol for a value render text only; flags remain the board-level
// icon in Settings.
private const val OFFICIAL = "file:///android_asset/cert-icons/official/"
private const val FLAGS = "file:///android_asset/cert-icons/flags/"

fun boardFlagUrl(region: String): String =
    FLAGS + region.lowercase() + ".svg"

fun certificationIconUrl(region: String, rating: String): String? {
    val file = OFFICIAL_RATING_ICONS[region.uppercase()]
        ?.get(rating.trim().uppercase())
        ?: return null
    return OFFICIAL + file
}

@Composable
fun CertificationIcon(region: String, rating: String, height: Dp = 18.dp) {
    val url = certificationIconUrl(region, rating) ?: return
    // The SVG aspect is unknown until it decodes; start at 2:1 (the widest
    // common board, e.g. MPA PG-13 at 1.97:1) and correct after decode so the
    // image box hugs the symbol instead of padding it with empty space.
    var aspect by remember(url) { mutableStateOf(2f) }
    AsyncImage(
        model = url,
        contentDescription = null,
        modifier = Modifier.height(height).width(height * aspect),
        onState = { state ->
            if (state is AsyncImagePainter.State.Success) {
                val size = state.painter.intrinsicSize
                if (size.height > 0f) {
                    aspect = size.width / size.height
                }
            }
        },
    )
}

@Composable
fun BoardFlagIcon(region: String) {
    AsyncImage(
        model = boardFlagUrl(region),
        contentDescription = null,
        modifier = Modifier.size(18.dp),
    )
}

/**
 * The board symbol (or its text fallback) on the shared light panel; log rows
 * and the film page use the same size and fallback rules.
 */
@Composable
fun CertificationBadge(region: String, rating: String, height: Dp = 34.dp) {
    Box(
        Modifier
            .clip(RoundedCornerShape(6.dp))
            .background(Color.White.copy(alpha = 0.18f))
            .padding(horizontal = 6.dp, vertical = 2.dp),
    ) {
        if (certificationIconUrl(region, rating) != null) {
            CertificationIcon(region, rating, height)
        } else {
            Text(
                rating,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
