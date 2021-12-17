window.BENCHMARK_DATA = {
  "lastUpdate": 1639750079029,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "str4d",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "cad50e7611cf49bb0bfa1d393cafee95c892b1d6",
          "message": "Merge pull request #265 from zcash/zcash_note_encryption-api-cleanups\n\nMigrate to latest `zcash_note_encryption` API",
          "timestamp": "2021-12-17T13:43:07Z",
          "tree_id": "f292d1a49afefc6288d6b5814f15ec3178d89412",
          "url": "https://github.com/zcash/orchard/commit/cad50e7611cf49bb0bfa1d393cafee95c892b1d6"
        },
        "date": 1639750077597,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 7218532497,
            "range": "± 50703284",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 7255159819,
            "range": "± 40813289",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 10597734159,
            "range": "± 38120069",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 13949878104,
            "range": "± 259890557",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 39133536,
            "range": "± 865923",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 39027086,
            "range": "± 1311906",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 44098013,
            "range": "± 957488",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 48972641,
            "range": "± 1849724",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1252572,
            "range": "± 12276",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 159769,
            "range": "± 2240",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1253993,
            "range": "± 9504",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 159997293,
            "range": "± 1894495",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 24559834,
            "range": "± 288221",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2824244,
            "range": "± 28596",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 24511698,
            "range": "± 308729",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2752366,
            "range": "± 52999",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 122539961,
            "range": "± 1540736",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 14112850,
            "range": "± 98050",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 123359934,
            "range": "± 1509976",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 13844884,
            "range": "± 131071",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 248153143,
            "range": "± 1284758",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 28003478,
            "range": "± 328062",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 246098970,
            "range": "± 1512929",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 27659288,
            "range": "± 406516",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 45554,
            "range": "± 613",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 167056,
            "range": "± 1680",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 182262,
            "range": "± 805",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 300253,
            "range": "± 1167",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 300338,
            "range": "± 1273",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 170371,
            "range": "± 1119",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 185303,
            "range": "± 1212",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 301154,
            "range": "± 3718",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 302156,
            "range": "± 3998",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 352313,
            "range": "± 4642",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 366174,
            "range": "± 6773",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 483030,
            "range": "± 7057",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 487651,
            "range": "± 5070",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 594579,
            "range": "± 8751",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 675145,
            "range": "± 6388",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}