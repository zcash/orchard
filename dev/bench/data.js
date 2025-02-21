window.BENCHMARK_DATA = {
  "lastUpdate": 1740101506259,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "kris@nutty.land",
            "name": "Kris Nuttycombe",
            "username": "nuttycom"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4ac248d05729e65d80ba8868d946d729f05c8525",
          "message": "Merge pull request #457 from zcash/release/v0.11.0\n\nRelease orchard version 0.11.0",
          "timestamp": "2025-02-20T18:18:51-07:00",
          "tree_id": "74133e9398f4dcd5fdc6c78b405632072ac1e8e4",
          "url": "https://github.com/zcash/orchard/commit/4ac248d05729e65d80ba8868d946d729f05c8525"
        },
        "date": 1740101505483,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2893940708,
            "range": "± 225280659",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2844753726,
            "range": "± 10851617",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4090737232,
            "range": "± 23509664",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5344188232,
            "range": "± 33653094",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24600293,
            "range": "± 719129",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24464332,
            "range": "± 323637",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 27693949,
            "range": "± 319396",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31483946,
            "range": "± 377539",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1476241,
            "range": "± 7330",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125574,
            "range": "± 456",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1472605,
            "range": "± 8232",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1330021582,
            "range": "± 1026369",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15582689,
            "range": "± 57668",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2131765,
            "range": "± 4894",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15558321,
            "range": "± 233970",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2096435,
            "range": "± 3009",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 77859429,
            "range": "± 163865",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10607275,
            "range": "± 96324",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 77751433,
            "range": "± 410013",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10427978,
            "range": "± 24582",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 155692019,
            "range": "± 291994",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21202750,
            "range": "± 37586",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 155485143,
            "range": "± 440375",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20844173,
            "range": "± 51310",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 463206,
            "range": "± 1285",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 487945,
            "range": "± 2416",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}