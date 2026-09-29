window.BENCHMARK_DATA = {
  "lastUpdate": 1790703591251,
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
          "id": "e000e9f8e236edc0c4f25c1be475e69039fa5551",
          "message": "Merge pull request #553 from zcash/dw/f08\n\nReuse Orchard domain objects, batch Merkle parent hashing, add benchmark harnesses",
          "timestamp": "2026-09-29T11:31:04-06:00",
          "tree_id": "579fa1e2ea6d27b58022237e78895c5a43b01fe9",
          "url": "https://github.com/zcash/orchard/commit/e000e9f8e236edc0c4f25c1be475e69039fa5551"
        },
        "date": 1790703589748,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 1503147546,
            "range": "± 173790147",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 1529631937,
            "range": "± 20228075",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 2337179840,
            "range": "± 138968448",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 2841013350,
            "range": "± 79222259",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 11552109,
            "range": "± 457234",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 11639158,
            "range": "± 719943",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 13318422,
            "range": "± 488352",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 15221218,
            "range": "± 913221",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 836599,
            "range": "± 53785",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 67413,
            "range": "± 3196",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 736505,
            "range": "± 20783",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 711461519,
            "range": "± 20172431",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 7445033,
            "range": "± 434431",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 729910,
            "range": "± 20166",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 7415566,
            "range": "± 85123",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 705482,
            "range": "± 25472",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 37103130,
            "range": "± 1660407",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 3582038,
            "range": "± 19234",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 37045359,
            "range": "± 1214731",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 3464525,
            "range": "± 32016",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 74216549,
            "range": "± 435263",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 7153206,
            "range": "± 7969",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 74088352,
            "range": "± 4634958",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 6938149,
            "range": "± 430266",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}