# Privacy Statement

This statement describes, in technical and binding terms, what NB Sound does with your data. It is enforced by the
code and by automated checks that run on every change; where a guarantee depends on a check, the check is named. The
detailed Spanish reference is [docs/audit/privacy.md](docs/audit/privacy.md).

## 1. Data sovereignty

* **Everything is local.** The library, playlists, listening history, profile, karaoke instrumentals, mashups and
  settings live in your user directories (`~/.local/share/nb-sound` and `~/.cache/nb-sound` on Linux,
  `%APPDATA%\NB Sound` on Windows, `~/Library/Application Support/NB Sound` on macOS). There is no server operated by
  the project that receives them.
* **Your music stays yours.** NB Sound writes in your music folder only when you import, edit a song's details or
  move your music to another folder. It never converts a file's format and never deletes a file without sending it to
  the system trash.
* **No account.** No feature requires registration, an email address, a licence key or a sign-in.
* **Portability.** A backup file holds the whole library state and restores it on another machine.

## 2. Zero telemetry

NB Sound contains **no** analytics, usage tracking, crash reporting, advertising identifiers or remote logging. The
module named `telemetry` writes local log files only, and it redacts content: logs never contain file paths, titles,
artists, albums or keys. This is enforced by `scripts/check-registro.mjs` on every change.

## 3. Network access

Every outgoing connection passes through one module, `src-tauri/src/net.rs`. Any HTTP client used elsewhere fails the
build checks (`scripts/check-network.mjs`). The gate enforces, in order:

1. **The global switch** "Use online services". When it is off, no connection to the internet is opened at all.
2. **Per-service consent.** Each service has its own switch in Settings, Privacy, and states what it receives.
3. **Allowed hosts.** Each service can only reach its own hosts.

| Service | What is sent | When |
|---|---|---|
| AcoustID | The acoustic fingerprint and length of a song, never the audio | When identifying a song |
| MusicBrainz, iTunes | Title, artist and album, or an identifier | When completing a song's details |
| Cover Art Archive, Deezer, Wikipedia, TheAudioDB | An album identifier or an artist or album name | When looking for covers and artist images |
| LRCLIB, lyrics.ovh | Title, artist, album and length | When looking for lyrics |
| AI provider (Anthropic or OpenAI) | The details of a song and its candidates | Only if you enable AI tie-breaking with your own key |
| Add-ons | Nothing from your library | When you press Install |
| Updates | Installed version and platform | When you check for updates |

## 4. Local network

* **NB Sound Mobile:** only phones paired with the QR code reach the library, over TLS with a pinned certificate.
  Removing a phone closes its access at once.
* **Party Mode:** only while it is on. Guests see the titles they search for and what is playing; they never get
  audio, history or paths.

## 5. Secrets

API keys for optional services are stored in the operating-system keyring, never in settings files, the database or
logs. If the system has no keyring, no key is stored.

## 6. Institutional use

This application does not report usage or listening activity to any institution. Institutional services, if any are
deployed, are separate products under their own agreements and cannot read local data through this application.

## 7. Changes

Any change to this statement is made in the same change as the code it describes, and is visible in the repository
history.
