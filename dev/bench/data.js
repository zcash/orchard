window.BENCHMARK_DATA = {
  "lastUpdate": 1734116464042,
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
          "id": "ccc90030d7b84d931931469fe1819e19e767ace1",
          "message": "Merge pull request #444 from zcash/pczt-user-address\n\npczt: Add output field for storing the user-facing address",
          "timestamp": "2024-12-13T11:47:13-07:00",
          "tree_id": "714f601c6fcc02d7abe0d90200be84073fc3f80c",
          "url": "https://github.com/zcash/orchard/commit/ccc90030d7b84d931931469fe1819e19e767ace1"
        },
        "date": 1734116462953,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2899031593,
            "range": "± 27745800",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2897142546,
            "range": "± 17577419",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4168258899,
            "range": "± 35975874",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5388836771,
            "range": "± 25686090",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25120514,
            "range": "± 575744",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25004662,
            "range": "± 570890",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28182144,
            "range": "± 626143",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31798606,
            "range": "± 406611",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1512277,
            "range": "± 6576",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 126359,
            "range": "± 1207",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1507283,
            "range": "± 7679",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1349939821,
            "range": "± 5426538",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15946343,
            "range": "± 47977",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2159713,
            "range": "± 6047",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15936183,
            "range": "± 577008",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2112091,
            "range": "± 75997",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 79669672,
            "range": "± 216728",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10739993,
            "range": "± 200329",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 79602002,
            "range": "± 185102",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10506178,
            "range": "± 30584",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 159337612,
            "range": "± 1775414",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21456852,
            "range": "± 626861",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 159138085,
            "range": "± 2247553",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20994555,
            "range": "± 87178",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465703,
            "range": "± 13234",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500654,
            "range": "± 852",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}