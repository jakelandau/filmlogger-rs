package ca.jakelandau.filmlogger.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import ca.jakelandau.filmlogger.R
import uniffi.film_logger_ffi.Format
import uniffi.film_logger_ffi.ScreenClass

enum class FormatFlag(val label: String, val iconRes: Int) {
    THREE_D("3D", R.drawable.ic_flag_3d),
    HFR("HFR", R.drawable.ic_flag_hfr),
    MOTION("Motion", R.drawable.ic_flag_motion),
}

/** Presentation flags for a theatre viewing; home viewings have none. */
fun formatFlags(format: Format?): List<FormatFlag> {
    if (format !is Format.Theatre) return emptyList()
    val motionScreen = format.screen is ScreenClass.FourDx || format.screen is ScreenClass.Ultra4Dx
    return buildList {
        if (format.threeD || motionScreen) add(FormatFlag.THREE_D)
        if (format.hfr) add(FormatFlag.HFR)
        if (format.dbox || motionScreen) add(FormatFlag.MOTION)
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun FormatFlagsRow(format: Format?) {
    val flags = formatFlags(format)
    if (flags.isEmpty()) return
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        for (flag in flags) FormatFlagChip(flag)
    }
}

@Composable
private fun FormatFlagChip(flag: FormatFlag) {
    Row(
        modifier = Modifier
            .background(MaterialTheme.colorScheme.surface, RoundedCornerShape(4.dp))
            .padding(horizontal = 5.dp, vertical = 2.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            painter = painterResource(flag.iconRes),
            contentDescription = null,
            modifier = Modifier.size(12.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.width(3.dp))
        Text(
            flag.label,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
