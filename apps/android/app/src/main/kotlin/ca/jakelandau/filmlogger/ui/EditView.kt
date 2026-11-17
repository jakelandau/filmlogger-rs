package ca.jakelandau.filmlogger.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
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
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.filmlogger_rs_ffi.Format
import uniffi.filmlogger_rs_ffi.FormatCatalog
import uniffi.filmlogger_rs_ffi.Logger
import uniffi.filmlogger_rs_ffi.LoggerException
import uniffi.filmlogger_rs_ffi.Rating
import uniffi.filmlogger_rs_ffi.Venue
import uniffi.filmlogger_rs_ffi.WatchLog
import uniffi.filmlogger_rs_ffi.formatFromParts
import uniffi.filmlogger_rs_ffi.formatParts

/**
 * Edit an existing watch log in place. The store derives `rewatch` and
 * `watch_count`, so they are shown read-only.
 */
@Composable
fun EditView(
    logger: Logger,
    catalog: FormatCatalog,
    log: WatchLog,
    onDone: () -> Unit,
    onDeleted: () -> Unit,
    onError: (String) -> Unit,
) {
    val scope = rememberCoroutineScope()
    var title by remember { mutableStateOf(log.title) }
    var venue by remember { mutableStateOf(log.venue) }
    var format by remember { mutableStateOf(log.format ?: formatFromParts(formatParts(null, log.venue), log.venue)) }
    var timezone by remember { mutableStateOf(log.timezone ?: TimeUtils.systemZone().id) }
    var watchedAtInput by remember {
        mutableStateOf(TimeUtils.toLocalInput(log.watchedAt, TimeUtils.zone(timezone)))
    }
    var rating by remember { mutableStateOf(log.rating) }
    var busy by remember { mutableStateOf(false) }
    var showDeleteConfirm by remember { mutableStateOf(false) }

    fun doSave() {
        val chosenRating = rating ?: run {
            onError("Choose a rating")
            return
        }
        val parsed = TimeUtils.fromLocalInput(watchedAtInput, TimeUtils.zone(timezone))
        if (parsed.isEmpty()) {
            onError("Invalid watched-at date or time zone")
            return
        }
        busy = true
        scope.launch {
            try {
                withContext(Dispatchers.IO) {
                    log.title = title
                    log.venue = venue
                    log.format = format
                    log.watchedAt = parsed
                    log.timezone = timezone
                    log.rating = chosenRating
                    logger.update(log)
                }
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
                withContext(Dispatchers.IO) { logger.delete(log.id) }
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
            .verticalScroll(rememberScrollState())
            .imePadding()
            .padding(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 16.dp),
    ) {
        Text(
            "Viewing ${log.watchCount ?: 1u} of this film" + if (log.rewatch) " · rewatch" else "",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(16.dp))

        OutlinedTextField(value = title, onValueChange = { title = it }, label = { Text("Title") }, modifier = Modifier.fillMaxWidth())
        Spacer(Modifier.height(12.dp))

        LogFields(
            catalog = catalog,
            venue = venue,
            onVenue = { venue = it },
            format = format,
            onFormat = { format = it },
            timezone = timezone,
            onTimezone = { timezone = it },
            watchedAtInput = watchedAtInput,
            onWatchedAt = { watchedAtInput = it },
            rating = rating,
            onRating = { rating = it },
        )
        Spacer(Modifier.height(12.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            OutlinedButton(
                onClick = { doSave() },
                enabled = !busy && rating != null,
                modifier = Modifier.weight(1f),
            ) {
                Text("Save")
            }
            OutlinedButton(
                onClick = onDone,
                enabled = !busy,
                modifier = Modifier.weight(1f),
            ) {
                Text("Cancel")
            }
        }
        Spacer(Modifier.height(24.dp))
        TextButton(
            onClick = { showDeleteConfirm = true },
            enabled = !busy,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(
                "Delete viewing…",
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.labelLarge,
            )
        }
        Spacer(Modifier.height(24.dp))
    }

    if (showDeleteConfirm) {
        AlertDialog(
            onDismissRequest = { if (!busy) showDeleteConfirm = false },
            title = { Text("Delete viewing?") },
            text = {
                Text("Delete “${log.title}”? This cannot be undone.")
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        showDeleteConfirm = false
                        doDelete()
                    },
                    enabled = !busy,
                ) {
                    Text("Delete", color = MaterialTheme.colorScheme.error)
                }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteConfirm = false }, enabled = !busy) {
                    Text("Cancel")
                }
            },
        )
    }
}
