package com.jakelandau.filmlogger.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import uniffi.film_logger_ffi.Stats

/** Aggregate statistics over the stored logs. */
@Composable
fun StatsView(stats: Stats) {
    LazyColumn(Modifier.padding(12.dp)) {
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

        item {
            Text("By year", style = MaterialTheme.typography.titleMedium)
            Spacer(Modifier.height(8.dp))
        }

        items(stats.viewingsByYear) { yearCount ->
            Row(Modifier.fillMaxWidth()) {
                Text(yearCount.year.toString())
                Spacer(Modifier.weight(1f))
                Text(yearCount.count.toInt().toString())
            }
        }

        item {
            Spacer(Modifier.height(16.dp))
            Text("Top films", style = MaterialTheme.typography.titleMedium)
            Spacer(Modifier.height(8.dp))
        }

        items(stats.topFilms) { film ->
            Row(Modifier.fillMaxWidth()) {
                Text(film.title)
                Spacer(Modifier.weight(1f))
                Text(film.count.toInt().toString())
            }
        }

        item { Spacer(Modifier.height(12.dp)) }
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
            Modifier.padding(12.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(value.toString(), style = MaterialTheme.typography.headlineSmall)
            Text(label, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}
