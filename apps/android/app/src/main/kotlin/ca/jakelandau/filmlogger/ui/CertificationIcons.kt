package ca.jakelandau.filmlogger.ui

import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage

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
fun CertificationIcon(region: String, rating: String, height: Dp = 18.dp, width: Dp = height) {
    val url = certificationIconUrl(region, rating) ?: return
    AsyncImage(
        model = url,
        contentDescription = null,
        modifier = Modifier.size(width = width, height = height),
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
