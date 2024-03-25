window.BENCHMARK_DATA = {
  "lastUpdate": 1711372079372,
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
          "id": "2b9c9a1deb66f8b20cd5c07fdd0cec87895f5c16",
          "message": "Merge pull request #424 from nuttycom/ivk_prepare\n\nAdd `IncomingViewingKey::prepare` convenience method.",
          "timestamp": "2024-03-25T06:54:10-06:00",
          "tree_id": "20cafcdfdb62c14b882aa4592e0f0eed9f2cb5c1",
          "url": "https://github.com/zcash/orchard/commit/2b9c9a1deb66f8b20cd5c07fdd0cec87895f5c16"
        },
        "date": 1711372078270,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2896144354,
            "range": "± 114461272",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2896078371,
            "range": "± 10138898",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4147993838,
            "range": "± 15684186",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5394643215,
            "range": "± 25096145",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25307466,
            "range": "± 536250",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25372369,
            "range": "± 465214",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28280743,
            "range": "± 1185281",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31969151,
            "range": "± 490021",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1525605,
            "range": "± 8822",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 128230,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1523184,
            "range": "± 8038",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1353920873,
            "range": "± 809769",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16135380,
            "range": "± 45078",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2174589,
            "range": "± 7043",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16110733,
            "range": "± 57771",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2150826,
            "range": "± 7816",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80619386,
            "range": "± 127284",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10820274,
            "range": "± 34163",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80462176,
            "range": "± 235414",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10699810,
            "range": "± 397731",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161266610,
            "range": "± 291140",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21628657,
            "range": "± 66398",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160986089,
            "range": "± 1515327",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21377958,
            "range": "± 68217",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465859,
            "range": "± 944",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500436,
            "range": "± 1071",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}