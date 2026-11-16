package ca.jakelandau.filmlogger.ui

/** Pure release-selection logic for the Android updater. */
data class ReleaseAsset(val name: String, val url: String)

object UpdateLogic {
    /** Numeric semver comparison; prerelease/build suffixes are ignored. */
    fun isNewer(remote: String, current: String): Boolean {
        val remoteParts = remote.removePrefix("v").substringBefore('-').split('.').map {
            it.toIntOrNull() ?: 0
        }
        val currentParts = current.removePrefix("v").substringBefore('-').split('.').map {
            it.toIntOrNull() ?: 0
        }
        for (index in 0 until maxOf(remoteParts.size, currentParts.size)) {
            val remotePart = remoteParts.getOrElse(index) { 0 }
            val currentPart = currentParts.getOrElse(index) { 0 }
            if (remotePart != currentPart) return remotePart > currentPart
        }
        return false
    }

    /** The APK asset from a GitHub release, if present. */
    fun selectApkAsset(assets: List<ReleaseAsset>): ReleaseAsset? =
        assets.firstOrNull { it.name.endsWith(".apk", ignoreCase = true) }
}
