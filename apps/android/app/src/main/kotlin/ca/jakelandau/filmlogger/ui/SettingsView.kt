package ca.jakelandau.filmlogger.ui

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuAnchorType
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.film_logger_ffi.Logger
import uniffi.film_logger_ffi.LoggerException
import uniffi.film_logger_ffi.RatingBoard
import uniffi.film_logger_ffi.Settings
import java.io.File

/**
 * App settings: the certification board shown in the list view, and
 * database export/import (the raw SQLite file).
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsView(
    logger: Logger,
    boards: List<RatingBoard>,
    onSettingsChanged: () -> Unit,
    onError: (String) -> Unit,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val scope = rememberCoroutineScope()
    var settings by remember { mutableStateOf<Settings?>(null) }
    var busy by remember { mutableStateOf(false) }
    var boardExpanded by remember { mutableStateOf(false) }

    suspend fun loadSettings() {
        settings = try {
            withContext(Dispatchers.IO) { logger.settings() }
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
                withContext(Dispatchers.IO) { logger.exportTo(file.absolutePath) }
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
                withContext(Dispatchers.IO) { logger.importFrom(file.absolutePath) }
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
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
    ) {
        Text("Certification board", style = MaterialTheme.typography.titleLarge)
        Spacer(Modifier.height(4.dp))
        Text(
            "Choose the board shown beside each log.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(8.dp))
        val board = settings?.selectedBoard
        val selectedName = board?.let { region ->
            boards.firstOrNull { it.region == region }?.name ?: region
        } ?: "Loading…"
        ExposedDropdownMenuBox(
            expanded = boardExpanded,
            onExpandedChange = {
                if (!busy && boards.isNotEmpty()) boardExpanded = !boardExpanded
            },
        ) {
            OutlinedTextField(
                value = selectedName,
                onValueChange = {},
                readOnly = true,
                label = { Text("Rating board") },
                trailingIcon = {
                    ExposedDropdownMenuDefaults.TrailingIcon(expanded = boardExpanded)
                },
                modifier = Modifier
                    .menuAnchor(MenuAnchorType.PrimaryNotEditable, enabled = !busy)
                    .fillMaxWidth(),
            )
            ExposedDropdownMenu(
                expanded = boardExpanded,
                onDismissRequest = { boardExpanded = false },
            ) {
                for (option in boards) {
                    DropdownMenuItem(
                        text = { Text(option.name, style = MaterialTheme.typography.bodyLarge) },
                        leadingIcon = { BoardFlagIcon(option.region) },
                        onClick = {
                            boardExpanded = false
                            if (option.region != board) {
                                scope.launch {
                                    try {
                                        withContext(Dispatchers.IO) {
                                            logger.setSelectedBoard(option.region)
                                        }
                                        onSettingsChanged()
                                        loadSettings()
                                    } catch (e: LoggerException) {
                                        onError(e.message ?: "board change failed")
                                    }
                                }
                            }
                        },
                    )
                }
            }
        }
        Spacer(Modifier.height(24.dp))

        Text("Database", style = MaterialTheme.typography.titleLarge)
        Spacer(Modifier.height(4.dp))
        Text(
            "Export a copy of the database, or replace the current log with a backup.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(10.dp))
        Button(
            onClick = { exportLauncher.launch("film-logger.db") },
            enabled = !busy,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Export database", style = MaterialTheme.typography.labelLarge)
        }
        Spacer(Modifier.height(10.dp))
        Button(
            onClick = { importLauncher.launch(arrayOf("application/octet-stream")) },
            enabled = !busy,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Import database", style = MaterialTheme.typography.labelLarge)
        }
        Spacer(Modifier.height(24.dp))

        Text("Licenses / credits", style = MaterialTheme.typography.titleLarge)
        Spacer(Modifier.height(4.dp))
        Text(
            "FilmLogger is built with these third-party libraries. " +
                "Transitive dependencies retain their own licenses.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        for (group in CREDIT_GROUPS) {
            Text(
                group.title,
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(top = 10.dp),
            )
            for (credit in group.items) {
                Text(
                    "• ${credit.name} — ${credit.license}",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        Spacer(Modifier.height(24.dp))
        Text("TMDB", style = MaterialTheme.typography.titleLarge)
        Spacer(Modifier.height(4.dp))
        Text(
            TMDB_ATTRIBUTION,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        TextButton(onClick = { uriHandler.openUri("https://www.themoviedb.org/") }) {
            Text("themoviedb.org", style = MaterialTheme.typography.labelLarge)
        }
    }
}
