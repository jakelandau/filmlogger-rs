package ca.jakelandau.filmlogger.ui

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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt
import uniffi.film_logger_ffi.CountEntry
import uniffi.film_logger_ffi.FormatCatalog
import uniffi.film_logger_ffi.Stats
import uniffi.film_logger_ffi.WatchLog

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

    val screenLabels = catalog.screens.flatMap { it.options }.associate { it.key to it.label }
    val mediumLabels = catalog.mediums.associate { it.key to it.label }
    val flagLabels = catalog.flagLabels.associate { it.key to it.label }

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

        breakdownValues(
            "Ratings",
            listOf(
                "👍 Liked" to stats.ratings.up.toInt(),
                "👎 Disliked" to stats.ratings.down.toInt(),
                "Unrated" to stats.ratings.unrated.toInt(),
            ),
        )
        breakdownValues(
            "Viewings by month",
            stats.viewingsByMonth
                .sortedByDescending { it.key }
                .map { it.key to it.count.toInt() },
        )
        breakdownValues(
            "Viewings by weekday",
            weekdayCounts(logs),
        )
        breakdownValues(
            "Viewings by hour",
            hourCounts(logs),
        )
        yearSection(stats)
        breakdownValues(
            "Viewings by projection family",
            countEntries(stats.viewingsByProjectionFamily) { familyLabel(it) },
        )
        breakdownValues(
            "Viewings by auditorium",
            countEntries(stats.viewingsByScreen) { screenLabels[it] ?: it },
        )
        breakdownValues(
            "Home viewings by medium",
            countEntries(stats.viewingsByMedium) { mediumLabels[it] ?: it },
        )
        breakdownValues(
            "Viewings by presentation flag",
            countEntries(stats.viewingsByFlag) { flagLabels[it] ?: it },
        )
        breakdownValues("Top genres", topGenres(stats))
        topFilmsSection(stats)
        item { Spacer(Modifier.height(12.dp)) }
    }
}

/** `Time watched: … · First: … · Last: …`, each half omitted when unknown. */
private fun metaLine(stats: Stats): String {
    if (stats.totalViewings == 0u) return ""
    val parts = buildList {
        if (stats.totalRuntimeMinutes > 0u) {
            add("Time watched: ${formatDuration(stats.totalRuntimeMinutes)}")
        }
        stats.firstViewedAt?.let { add("First: ${TimeUtils.display(it)}") }
        stats.lastViewedAt?.let { add("Last: ${TimeUtils.display(it)}") }
    }
    return parts.joinToString(" · ")
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
    "home" -> "Home"
    else -> key
}

/** One heading plus a header row and one row per year, descending. */
private fun LazyListScope.yearSection(stats: Stats) {
    if (stats.viewingsByYear.isEmpty()) return
    item {
        Text("Viewings by year", style = MaterialTheme.typography.titleLarge)
        Spacer(Modifier.height(8.dp))
        HeaderRow("Year", "Viewings")
    }
    items(stats.viewingsByYear.reversed()) { yearCount ->
        KeyValueRow(yearCount.year.toString(), yearCount.count.toString())
    }
    item { Spacer(Modifier.height(12.dp)) }
}

/** One heading plus a header row and the ten most-watched films. */
private fun LazyListScope.topFilmsSection(stats: Stats) {
    val films = stats.topFilms.take(10)
    if (films.isEmpty()) return
    item {
        Text("Most watched", style = MaterialTheme.typography.titleLarge)
        Spacer(Modifier.height(8.dp))
        HeaderRow("Film", "Times")
    }
    items(films) { film -> KeyValueRow(film.title, film.count.toString()) }
    item { Spacer(Modifier.height(12.dp)) }
}

@Composable
private fun HeaderRow(left: String, right: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
    ) {
        Text(
            left,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.weight(1f))
        Text(
            right,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun KeyValueRow(left: String, right: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 6.dp),
    ) {
        Text(left, style = MaterialTheme.typography.bodyMedium)
        Spacer(Modifier.weight(1f))
        Text(right, style = MaterialTheme.typography.bodyMedium)
    }
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
