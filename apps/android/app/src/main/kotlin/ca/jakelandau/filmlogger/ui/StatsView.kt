package ca.jakelandau.filmlogger.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt
import uniffi.filmlogger_rs_ffi.CountEntry
import uniffi.filmlogger_rs_ffi.FormatCatalog
import uniffi.filmlogger_rs_ffi.Stats
import uniffi.filmlogger_rs_ffi.WatchLog

/** Slice colors, shared with the desktop stats screen. */
private val FAMILY_COLORS = mapOf(
    "dci" to Color(0xFFD489A4),
    "imax" to Color(0xFF8FC4E8),
    "film" to Color(0xFFFFB27A),
)
private val FALLBACK_COLOR = Color(0xFF9D95A3)
private val RATING_UP_COLOR = Color(0xFF8FE3C0)
private val RATING_DOWN_COLOR = Color(0xFFE5484D)

/** One pie slice: label, count, and slice color. */
private data class Slice(val label: String, val count: Int, val color: Color)

/** The switchable timeline views, matching the desktop buttons. */
private enum class TimelineMode(val label: String) {
    YEAR("Year"),
    MONTH("Month"),
    WEEKDAY("Weekday"),
}

/**
 * Aggregate statistics over the stored logs. Section order, labels, and
 * values match the desktop Stats tab exactly.
 */
@Composable
fun StatsView(stats: Stats?, catalog: FormatCatalog, logs: List<WatchLog>) {
    if (stats == null) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text("Loading…", style = MaterialTheme.typography.bodyMedium)
        }
        return
    }

    val mediumLabels = catalog.mediums.associate { it.key to it.label }
    val flagLabels = catalog.flagLabels.associate { it.key to it.label }
    val ratings = ratingSlices(stats)
    val families = familySlices(stats)

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 16.dp),
    ) {
        item {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                StatCard(Modifier.weight(1f), stats.totalViewings.toString(), "Total viewings")
                StatCard(Modifier.weight(1f), stats.distinctFilms.toString(), "Distinct films")
            }
            Spacer(Modifier.height(8.dp))
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                StatCard(Modifier.weight(1f), stats.viewingsByVenue.theatre.toString(), "Theatre")
                StatCard(Modifier.weight(1f), stats.viewingsByVenue.home.toString(), "Home")
            }
            Spacer(Modifier.height(8.dp))
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                StatCard(Modifier.weight(1f), stats.rewatches.toString(), "Rewatches")
                StatCard(Modifier.weight(1f), rewatchRate(stats), "Rewatch rate")
            }
            val meta = metaLine(stats)
            if (meta.isNotEmpty()) {
                Spacer(Modifier.height(12.dp))
                Text(
                    meta,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Spacer(Modifier.height(16.dp))
        }

        if (ratings.isNotEmpty()) {
            item { PieSection("Ratings", ratings) }
        }
        item { TimelineSection(stats, logs) }
        breakdownValues("Viewings by time of day", timeOfDayCounts(logs))
        if (families.isNotEmpty()) {
            item { PieSection("Viewings by projection family", families) }
        }
        breakdownValues(
            "Home viewings by medium",
            countEntries(stats.viewingsByMedium) { mediumLabels[it] ?: it },
        )
        breakdownValues(
            "Viewings by presentation flag",
            countEntries(stats.viewingsByFlag) { flagLabels[it] ?: it },
        )
        breakdownValues("Top genres", topGenres(stats))
        item { Spacer(Modifier.height(12.dp)) }
    }
}

/** `Time watched: …`, omitted when no runtime is known. */
private fun metaLine(stats: Stats): String {
    if (stats.totalViewings == 0u || stats.totalRuntimeMinutes == 0u) return ""
    return "Time watched: ${formatDuration(stats.totalRuntimeMinutes)}"
}

/** `Xh Ym`, `Xh` on the hour, `Ym` under an hour. */
private fun formatDuration(minutes: UInt): String {
    val total = minutes.toLong()
    val hours = total / 60
    val rest = total % 60
    return when {
        hours == 0L -> "${rest}m"
        rest == 0L -> "${hours}h"
        else -> "${hours}h ${rest}m"
    }
}

private fun rewatchRate(stats: Stats): String {
    if (stats.totalViewings == 0u) return "—"
    val pct = (stats.rewatches.toDouble() / stats.totalViewings.toDouble() * 100).roundToInt()
    return "$pct%"
}

/** Liked/disliked slices; both are listed once anything is rated. */
private fun ratingSlices(stats: Stats): List<Slice> {
    if (stats.ratings.up + stats.ratings.down == 0u) return emptyList()
    return listOf(
        Slice("👍 Liked", stats.ratings.up.toInt(), RATING_UP_COLOR),
        Slice("👎 Disliked", stats.ratings.down.toInt(), RATING_DOWN_COLOR),
    )
}

/** One slice per projection family, count descending then key ascending. */
private fun familySlices(stats: Stats): List<Slice> = stats.viewingsByProjectionFamily
    .sortedWith(compareByDescending<CountEntry> { it.count }.thenBy { it.key })
    .map { entry ->
        Slice(familyLabel(entry.key), entry.count.toInt(), FAMILY_COLORS[entry.key] ?: FALLBACK_COLOR)
    }

/** Count-descending, key-ascending `(label, count)` entries. */
private fun countEntries(entries: List<CountEntry>, label: (String) -> String): List<Pair<String, Int>> =
    entries
        .sortedWith(compareByDescending<CountEntry> { it.count }.thenBy { it.key })
        .map { label(it.key) to it.count.toInt() }

/** Genres by viewing count, count descending then name ascending, first 10. */
private fun topGenres(stats: Stats): List<Pair<String, Int>> = stats.viewingsByGenre
    .sortedWith(compareByDescending<CountEntry> { it.count }.thenBy { it.key })
    .take(10)
    .map { it.key to it.count.toInt() }

private fun familyLabel(key: String): String = when (key) {
    "dci" -> "Digital (DCI)"
    "imax" -> "IMAX"
    "film" -> "Film"
    else -> key
}

/** A pie plus legend, rendered as one list item. */
@Composable
private fun PieSection(title: String, slices: List<Slice>) {
    val total = slices.sumOf { it.count }
    if (total == 0) return
    Text(title, style = MaterialTheme.typography.titleLarge)
    Spacer(Modifier.height(8.dp))
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Canvas(Modifier.size(140.dp)) {
            var start = -90f
            for (slice in slices) {
                if (slice.count == 0) continue
                val sweep = 360f * slice.count / total
                drawArc(
                    color = slice.color,
                    startAngle = start,
                    sweepAngle = sweep,
                    useCenter = true,
                )
                start += sweep
            }
        }
        Spacer(Modifier.width(20.dp))
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            for (slice in slices) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Box(
                        Modifier
                            .size(10.dp)
                            .clip(RoundedCornerShape(2.dp))
                            .background(slice.color),
                    )
                    Spacer(Modifier.width(8.dp))
                    Text(
                        slice.label,
                        modifier = Modifier.weight(1f),
                        style = MaterialTheme.typography.bodyMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        slice.count.toString(),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
    Spacer(Modifier.height(12.dp))
}

/** Year, month, and weekday viewings in one section, newest first. */
@Composable
private fun TimelineSection(stats: Stats, logs: List<WatchLog>) {
    val yearEntries = stats.viewingsByYear
        .sortedByDescending { it.year }
        .map { it.year.toString() to it.count.toInt() }
    // The core orders months oldest first; show newest first like years.
    val monthEntries = stats.viewingsByMonth.reversed().map { it.key to it.count.toInt() }
    val weekdayEntries = weekdayCounts(logs)
    if (yearEntries.isEmpty() && monthEntries.isEmpty() && weekdayEntries.isEmpty()) return

    var mode by remember { mutableStateOf(TimelineMode.YEAR) }
    val entries = when (mode) {
        TimelineMode.YEAR -> yearEntries
        TimelineMode.MONTH -> monthEntries
        TimelineMode.WEEKDAY -> weekdayEntries
    }
    Text("Viewings over time", style = MaterialTheme.typography.titleLarge)
    Spacer(Modifier.height(8.dp))
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        for (option in TimelineMode.entries) {
            FilterChip(
                selected = mode == option,
                onClick = { mode = option },
                label = { Text(option.label) },
            )
        }
    }
    Spacer(Modifier.height(8.dp))
    val max = entries.maxOfOrNull { it.second } ?: 0
    Column(Modifier.fillMaxWidth()) {
        for ((label, count) in entries) BreakdownRow(label, count, max)
    }
    Spacer(Modifier.height(12.dp))
}

/** Add one breakdown section (title plus proportional-bar rows) to the list. */
private fun LazyListScope.breakdownValues(title: String, entries: List<Pair<String, Int>>) {
    if (entries.isEmpty()) return
    val max = entries.maxOf { it.second }
    item {
        Text(title, style = MaterialTheme.typography.titleLarge)
        Spacer(Modifier.height(8.dp))
        Column(Modifier.fillMaxWidth()) {
            for ((label, count) in entries) BreakdownRow(label, count, max)
        }
        Spacer(Modifier.height(12.dp))
    }
}

@Composable
private fun BreakdownRow(label: String, count: Int, max: Int) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 2.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            label,
            modifier = Modifier.width(150.dp),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Box(
            modifier = Modifier
                .weight(1f)
                .height(12.dp)
                .clip(RoundedCornerShape(999.dp))
                .background(MaterialTheme.colorScheme.surface),
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth(if (max == 0) 0f else count.toFloat() / max)
                    .fillMaxHeight()
                    .background(MaterialTheme.colorScheme.primary),
            )
        }
        Spacer(Modifier.width(8.dp))
        Text(count.toString(), style = MaterialTheme.typography.bodyMedium)
    }
}

@Composable
private fun StatCard(modifier: Modifier, value: String, label: String) {
    Surface(
        modifier = modifier,
        shape = MaterialTheme.shapes.medium,
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Column(
            Modifier.padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(value, style = MaterialTheme.typography.headlineMedium)
            Spacer(Modifier.height(2.dp))
            Text(
                label,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
