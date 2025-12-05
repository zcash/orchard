window.BENCHMARK_DATA = {
  "lastUpdate": 1764950886970,
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
          "id": "ddbff98691b42c68813a4db3bed2e740bc6cc587",
          "message": "Merge pull request #473 from zcash/more-release-prep\n\nMore release preparations",
          "timestamp": "2025-12-05T08:55:19-07:00",
          "tree_id": "7237700619c034c43a5b7c93c5527050cac66961",
          "url": "https://github.com/zcash/orchard/commit/ddbff98691b42c68813a4db3bed2e740bc6cc587"
        },
        "date": 1764950885490,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2695384975,
            "range": "± 256192969",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2685183645,
            "range": "± 16546146",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3874784390,
            "range": "± 17465424",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5050088804,
            "range": "± 37480425",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 21131666,
            "range": "± 409548",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 21169622,
            "range": "± 179841",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 24599103,
            "range": "± 177488",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 27934152,
            "range": "± 227440",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1479492,
            "range": "± 6198",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125793,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1477052,
            "range": "± 8598",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1330720943,
            "range": "± 1923171",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15632124,
            "range": "± 27353",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2137597,
            "range": "± 19210",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15605994,
            "range": "± 42772",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2101463,
            "range": "± 3365",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78121388,
            "range": "± 153513",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10628999,
            "range": "± 10976",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 78001294,
            "range": "± 90951",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10458372,
            "range": "± 15691",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156180547,
            "range": "± 325855",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21251135,
            "range": "± 31925",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 155988998,
            "range": "± 403157",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20905269,
            "range": "± 30615",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 463760,
            "range": "± 1392",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488770,
            "range": "± 765",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}