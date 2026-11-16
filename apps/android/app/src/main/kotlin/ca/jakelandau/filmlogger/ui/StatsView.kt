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
import uniffi.film_logger_ffi.CountEntry
import uniffi.film_logger_ffi.FormatCatalog
import uniffi.film_logger_ffi.Stats

/** Aggregate statistics over the stored logs, including format breakdowns. */
@Composable
fun StatsView(stats: Stats, catalog: FormatCatalog) {
    val screenLabels = catalog.screens.flatMap { it.options }.associate { it.key to it.label }
    val mediumLabels = catalog.mediums.associate { it.key to it.label }
    val flagLabels = catalog.flagLabels.associate { it.key to it.label }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
    ) {
        item {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                StatCard(Modifier.weight(1f), stats.totalViewings.toInt(), "viewings")
                StatCard(Modifier.weight(1f), stats.distinctFilms.toInt(), "films")
            }
            Spacer(Modifier.height(8.dp))
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                StatCard(Modifier.weight(1f), stats.viewingsByVenue.theatre.toInt(), "theatre")
                StatCard(Modifier.weight(1f), stats.viewingsByVenue.home.toInt(), "home")
            }
            Spacer(Modifier.height(8.dp))
            StatCard(Modifier.fillMaxWidth(), stats.rewatches.toInt(), "rewatches")
            Spacer(Modifier.height(16.dp))
        }

        breakdownSection("By month", stats.viewingsByMonth, descendingKeys = true) { it }
        breakdownSection("By projection family", stats.viewingsByProjectionFamily) { familyLabel(it) }
        breakdownSection("By auditorium", stats.viewingsByScreen) { screenLabels[it] ?: it }
        breakdownSection("Home medium", stats.viewingsByMedium) { mediumLabels[it] ?: it }
        breakdownSection("By presentation flag", stats.viewingsByFlag) { flagLabels[it] ?: it }

        item {
            Text("By year", style = MaterialTheme.typography.titleLarge)
            Spacer(Modifier.height(8.dp))
        }

        items(stats.viewingsByYear) { yearCount ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 6.dp),
            ) {
                Text(yearCount.year.toString(), style = MaterialTheme.typography.bodyMedium)
                Spacer(Modifier.weight(1f))
                Text(yearCount.count.toInt().toString(), style = MaterialTheme.typography.bodyMedium)
            }
        }

        item {
            Spacer(Modifier.height(20.dp))
            Text("Top films", style = MaterialTheme.typography.titleLarge)
            Spacer(Modifier.height(8.dp))
        }

        items(stats.topFilms) { film ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 6.dp),
            ) {
                Text(film.title, style = MaterialTheme.typography.bodyMedium)
                Spacer(Modifier.weight(1f))
                Text(film.count.toInt().toString(), style = MaterialTheme.typography.bodyMedium)
            }
        }

        item { Spacer(Modifier.height(12.dp)) }
    }
}

private fun familyLabel(key: String): String = when (key) {
    "dci" -> "Digital (DCI)"
    "imax" -> "IMAX"
    "film" -> "Film"
    "home" -> "Home"
    else -> key
}

/** Add one breakdown section (title plus proportional-bar rows) to the list. */
private fun androidx.compose.foundation.lazy.LazyListScope.breakdownSection(
    title: String,
    entries: List<CountEntry>,
    descendingKeys: Boolean = false,
    label: (String) -> String,
) {
    if (entries.isEmpty()) return
    val ordered = if (descendingKeys) {
        entries.sortedByDescending { it.key }
    } else {
        entries.sortedByDescending { it.count }
    }
    item {
        Text(title, style = MaterialTheme.typography.titleLarge)
        Spacer(Modifier.height(8.dp))
        Column(Modifier.fillMaxWidth()) {
            for (entry in ordered) BreakdownRow(label(entry.key), entry.count.toInt(), ordered.maxOf { it.count.toInt() })
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
private fun StatCard(modifier: Modifier, value: Int, label: String) {
    Surface(
        modifier = modifier,
        shape = MaterialTheme.shapes.medium,
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Column(
            Modifier.padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(value.toString(), style = MaterialTheme.typography.headlineMedium)
            Spacer(Modifier.height(2.dp))
            Text(
                label,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
