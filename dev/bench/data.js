window.BENCHMARK_DATA = {
  "lastUpdate": 1723753015245,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira-Emma Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ce1be620440340d416678db87bbdcfb145e002df",
          "message": "Merge pull request #435 from zcash/update_visibility\n\nUpdate `visibility` crate version to one that is properly MIT/Apache licensed.",
          "timestamp": "2024-08-15T20:58:17+01:00",
          "tree_id": "e59ee35070303d59bc83ca27d22dbe447f5b0010",
          "url": "https://github.com/zcash/orchard/commit/ce1be620440340d416678db87bbdcfb145e002df"
        },
        "date": 1723753013784,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2896419699,
            "range": "± 21748910",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2903533061,
            "range": "± 8701407",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4139389769,
            "range": "± 18940259",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5398156971,
            "range": "± 16121014",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25074763,
            "range": "± 564688",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24796517,
            "range": "± 419988",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28118125,
            "range": "± 524514",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31565694,
            "range": "± 464779",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1526235,
            "range": "± 9105",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127519,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1525995,
            "range": "± 5326",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1355117878,
            "range": "± 1040850",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16134800,
            "range": "± 58727",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2178949,
            "range": "± 9243",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16100000,
            "range": "± 39141",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2141683,
            "range": "± 7801",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80585649,
            "range": "± 193119",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10841786,
            "range": "± 43458",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80442552,
            "range": "± 298521",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10657837,
            "range": "± 12986",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161223443,
            "range": "± 157541",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21669710,
            "range": "± 55663",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160902510,
            "range": "± 390078",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21296848,
            "range": "± 17703",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 467895,
            "range": "± 2623",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 502941,
            "range": "± 6518",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}