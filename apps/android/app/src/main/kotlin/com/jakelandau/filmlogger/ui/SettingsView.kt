package com.jakelandau.filmlogger.ui

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import uniffi.film_logger_ffi.Logger
import uniffi.film_logger_ffi.LoggerException
import uniffi.film_logger_ffi.RatingBoard
import uniffi.film_logger_ffi.Settings
import java.io.File

/**
 * App settings: the certification board shown in the list view, and
 * database export/import (the raw SQLite file).
 */
@Composable
fun SettingsView(
    logger: Logger,
    onSettingsChanged: () -> Unit,
    onError: (String) -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var settings by remember { mutableStateOf<Settings?>(null) }
    var busy by remember { mutableStateOf(false) }

    fun loadSettings() {
        settings = try {
            logger.settings()
        } catch (e: LoggerException) {
            onError(e.message ?: "settings failed")
            null
        }
    }

    LaunchedEffect(Unit) {
        loadSettings()
    }

    fun exportToUri(uri: Uri?) {
        if (uri == null) return
        busy = true
        scope.launch {
            try {
                val file = File(context.cacheDir, "film-logger.db")
                logger.exportTo(file.absolutePath)
                context.contentResolver.openOutputStream(uri)?.use { output ->
                    file.inputStream().use { it.copyTo(output) }
                }
            } catch (e: Exception) {
                onError(e.message ?: "export failed")
            } finally {
                busy = false
            }
        }
    }

    fun importFromUri(uri: Uri?) {
        if (uri == null) return
        busy = true
        scope.launch {
            try {
                val file = File(context.cacheDir, "film-logger-import.db")
                context.contentResolver.openInputStream(uri)?.use { input ->
                    file.outputStream().use { input.copyTo(it) }
                }
                logger.importFrom(file.absolutePath)
                onSettingsChanged()
            } catch (e: Exception) {
                onError(e.message ?: "import failed")
            } finally {
                busy = false
            }
        }
    }

    val exportLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.CreateDocument("application/octet-stream"),
    ) { uri -> exportToUri(uri) }

    val importLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.OpenDocument(),
    ) { uri -> importFromUri(uri) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(12.dp),
    ) {
        Text("Certification board", style = MaterialTheme.typography.titleMedium)
        Spacer(Modifier.height(8.dp))
        val board = settings?.selectedBoard
        for (option in RatingBoard.entries) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                RadioButton(
                    selected = board == option,
                    enabled = !busy,
                    onClick = {
                        scope.launch {
                            try {
                                logger.setSelectedBoard(option)
                                onSettingsChanged()
                                loadSettings()
                            } catch (e: LoggerException) {
                                onError(e.message ?: "board change failed")
                            }
                        }
                    },
                )
                Text(option.name)
            }
            Spacer(Modifier.height(4.dp))
        }
        Spacer(Modifier.height(16.dp))

        Text("Database", style = MaterialTheme.typography.titleMedium)
        Spacer(Modifier.height(8.dp))
        Button(onClick = { exportLauncher.launch("film-logger.db") }, enabled = !busy, modifier = Modifier.fillMaxWidth()) {
            Text("Export database")
        }
        Spacer(Modifier.height(8.dp))
        Button(
            onClick = { importLauncher.launch(arrayOf("application/octet-stream")) },
            enabled = !busy,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Import database")
        }
        Spacer(Modifier.height(8.dp))
        Text(
            "Export saves the database file; import replaces the stored log with the selected file.",
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
