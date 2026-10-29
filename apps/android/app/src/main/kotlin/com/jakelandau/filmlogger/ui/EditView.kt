package com.jakelandau.filmlogger.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import uniffi.film_logger_ffi.Format
import uniffi.film_logger_ffi.Logger
import uniffi.film_logger_ffi.LoggerException
import uniffi.film_logger_ffi.Rating
import uniffi.film_logger_ffi.Venue
import uniffi.film_logger_ffi.WatchLog

/**
 * Edit an existing watch log in place.
 */
@Composable
fun EditView(
    logger: Logger,
    log: WatchLog,
    onDone: () -> Unit,
    onDeleted: () -> Unit,
    onError: (String) -> Unit,
) {
    val scope = rememberCoroutineScope()
    var title by remember { mutableStateOf(log.title) }
    var venue by remember { mutableStateOf(log.venue) }
    var format by remember { mutableStateOf(log.format ?: FormatText.build(FormatParts.forVenue(log.venue))) }
    var watchedAtInput by remember { mutableStateOf(TimeUtils.toLocalInput(log.watchedAt)) }
    var notes by remember { mutableStateOf(log.notes ?: "") }
    var rating by remember { mutableStateOf(log.rating) }
    var rewatch by remember { mutableStateOf(log.rewatch) }
    var watchCount by remember { mutableStateOf(log.watchCount?.toString() ?: "") }
    var busy by remember { mutableStateOf(false) }

    fun doSave() {
        busy = true
        scope.launch {
            try {
                log.title = title
                log.venue = venue
                log.format = format
                val parsed = TimeUtils.fromLocalInput(watchedAtInput)
                log.watchedAt = if (parsed.isEmpty()) log.watchedAt else parsed
                log.notes = notes.ifBlank { null }
                log.rating = rating
                log.rewatch = rewatch
                log.watchCount = watchCount.toIntOrNull()
                logger.update(log)
                onDone()
            } catch (e: LoggerException) {
                onError(e.message ?: "save failed")
            } finally {
                busy = false
            }
        }
    }

    fun doDelete() {
        busy = true
        scope.launch {
            try {
                logger.delete(log.id)
                onDeleted()
            } catch (e: LoggerException) {
                onError(e.message ?: "delete failed")
            } finally {
                busy = false
            }
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(12.dp),
    ) {
        Text(log.title.ifBlank { "Untitled" }, style = MaterialTheme.typography.titleLarge)
        val year = log.releaseYear
        Text(year?.let { "($it)" } ?: "", color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text("id ${log.id} · watched ${TimeUtils.display(log.watchedAt)}")
        Spacer(Modifier.height(12.dp))

        OutlinedTextField(value = title, onValueChange = { title = it }, label = { Text("Title") }, modifier = Modifier.fillMaxWidth())
        Spacer(Modifier.height(12.dp))

        LogFields(
            venue = venue,
            onVenue = { venue = it },
            format = format,
            onFormat = { format = it },
            watchedAtInput = watchedAtInput,
            onWatchedAt = { watchedAtInput = it },
            rating = rating,
            onRating = { rating = it },
            rewatch = rewatch,
            onRewatch = { rewatch = it },
            watchCount = watchCount,
            onWatchCount = { watchCount = it },
            notes = notes,
            onNotes = { notes = it },
        )
        Spacer(Modifier.height(12.dp))

        TextButton(onClick = { doDelete() }, enabled = !busy) { Text("Delete") }
        OutlinedButton(onClick = { doSave() }, enabled = !busy, modifier = Modifier.fillMaxWidth()) {
            Text("Save")
        }
    }
}
