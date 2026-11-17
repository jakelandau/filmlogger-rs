package ca.jakelandau.filmlogger.ui

import android.net.Uri
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuAnchorType
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
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
    onImported: () -> Unit,
    onError: (String) -> Unit,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val scope = rememberCoroutineScope()
    var settings by remember { mutableStateOf<Settings?>(null) }
    var busy by remember { mutableStateOf(false) }
    var boardExpanded by remember { mutableStateOf(false) }
    var showLicenses by remember { mutableStateOf(false) }
    var updateBusy by remember { mutableStateOf(false) }
    var updateStatus by remember { mutableStateOf<String?>(null) }
    var ioStatus by remember { mutableStateOf<String?>(null) }
    var pendingImport by remember { mutableStateOf<Uri?>(null) }

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

    BackHandler(enabled = showLicenses) {
        showLicenses = false
    }

    fun exportToUri(uri: Uri?) {
        if (uri == null) return
        busy = true
        ioStatus = null
        scope.launch {
            try {
                val file = File(context.cacheDir, "film-logger.db")
                withContext(Dispatchers.IO) { logger.exportTo(file.absolutePath) }
                context.contentResolver.openOutputStream(uri)?.use { output ->
                    file.inputStream().use { it.copyTo(output) }
                }
                ioStatus = "Exported database."
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
        ioStatus = null
        scope.launch {
            try {
                val file = File(context.cacheDir, "film-logger-import.db")
                context.contentResolver.openInputStream(uri)?.use { input ->
                    file.outputStream().use { input.copyTo(it) }
                }
                withContext(Dispatchers.IO) { logger.importFrom(file.absolutePath) }
                ioStatus = "Imported database."
                onImported()
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

    // Any document type: a desktop-exported `.db` synced through Drive or
    // Downloads is often reported as `application/x-sqlite3` or as binary
    // data, so the octet-stream filter alone would hide it.
    val importLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.OpenDocument(),
    ) { uri -> pendingImport = uri }

    pendingImport?.let { uri ->
        AlertDialog(
            onDismissRequest = { pendingImport = null },
            title = { Text("Import database") },
            text = { Text("Replace all current data?") },
            confirmButton = {
                TextButton(onClick = {
                    pendingImport = null
                    importFromUri(uri)
                }) {
                    Text("Import")
                }
            },
            dismissButton = {
                TextButton(onClick = { pendingImport = null }) {
                    Text("Cancel")
                }
            },
        )
    }

    if (showLicenses) {
        LicensesPage(onBack = { showLicenses = false })
        return
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 16.dp),
    ) {
        Text("Rating board", style = MaterialTheme.typography.titleLarge)
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
            onClick = { importLauncher.launch(arrayOf("*/*")) },
            enabled = !busy,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Import database", style = MaterialTheme.typography.labelLarge)
        }
        ioStatus?.let {
            Spacer(Modifier.height(6.dp))
            Text(
                it,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Spacer(Modifier.height(24.dp))

        Text("Updates", style = MaterialTheme.typography.titleLarge)
        Spacer(Modifier.height(8.dp))
        Button(
            onClick = {
                updateBusy = true
                scope.launch {
                    val result = withContext(Dispatchers.IO) {
                        runCatching { Updater.checkAndInstall(context) }
                            .getOrElse { Updater.Result(it.message ?: "Update check failed") }
                    }
                    updateStatus = result.message
                    updateBusy = false
                }
            },
            enabled = !updateBusy,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(
                if (updateBusy) "Checking…" else "Check for Updates",
                style = MaterialTheme.typography.labelLarge,
            )
        }
        updateStatus?.let {
            Spacer(Modifier.height(6.dp))
            Text(
                it,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        Spacer(Modifier.height(24.dp))
        Text("TMDB", style = MaterialTheme.typography.titleLarge)
        Spacer(Modifier.height(8.dp))
        Surface(
            modifier = Modifier.fillMaxWidth(),
            shape = MaterialTheme.shapes.medium,
            color = MaterialTheme.colorScheme.surfaceVariant,
        ) {
            Column(Modifier.padding(16.dp)) {
                AsyncImage(
                    model = "file:///android_asset/tmdb-logo.svg",
                    contentDescription = "The Movie Database (TMDB)",
                    contentScale = ContentScale.Fit,
                    modifier = Modifier
                        .width(220.dp)
                        .height(18.dp),
                )
                Spacer(Modifier.height(10.dp))
                Text(
                    TMDB_ATTRIBUTION,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                TextButton(
                    onClick = { uriHandler.openUri("https://www.themoviedb.org/") },
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(48.dp),
                ) {
                    Text(
                        "themoviedb.org",
                        modifier = Modifier.fillMaxWidth(),
                        textAlign = TextAlign.Center,
                        style = MaterialTheme.typography.titleMedium,
                    )
                }
            }
        }

        Spacer(Modifier.height(16.dp))
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .clickable { showLicenses = true },
            shape = MaterialTheme.shapes.medium,
            color = MaterialTheme.colorScheme.surfaceVariant,
        ) {
            Row(
                modifier = Modifier.padding(16.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(Modifier.weight(1f)) {
                    Text("Licenses & credits", style = MaterialTheme.typography.titleMedium)
                    Text(
                        "Third-party libraries and assets",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                Text(
                    "›",
                    style = MaterialTheme.typography.headlineSmall,
                    color = MaterialTheme.colorScheme.primary,
                )
            }
        }
        Spacer(Modifier.height(24.dp))
    }
}

@Composable
private fun LicensesPage(onBack: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 24.dp),
    ) {
        TextButton(onClick = onBack) {
            Text("← Settings", style = MaterialTheme.typography.labelLarge)
        }
        Text("Licenses & credits", style = MaterialTheme.typography.headlineSmall)
        Spacer(Modifier.height(4.dp))
        Text(
            "FilmLogger is built with these third-party libraries. " +
                "Transitive dependencies retain their own licenses.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(12.dp))

        for (group in CREDIT_GROUPS) {
            Surface(
                modifier = Modifier.fillMaxWidth(),
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surfaceVariant,
            ) {
                Column(Modifier.padding(14.dp)) {
                    Text(group.title, style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(6.dp))
                    group.items.forEachIndexed { index, credit ->
                        if (index > 0) {
                            HorizontalDivider(
                                modifier = Modifier.padding(vertical = 6.dp),
                                color = MaterialTheme.colorScheme.outlineVariant,
                            )
                        }
                        Text(credit.name, style = MaterialTheme.typography.bodyMedium)
                        Text(
                            credit.license,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
            Spacer(Modifier.height(10.dp))
        }
        Spacer(Modifier.height(12.dp))
    }
}
