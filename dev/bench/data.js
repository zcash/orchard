window.BENCHMARK_DATA = {
  "lastUpdate": 1740098621491,
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
          "id": "319d0de29a683af9027bed361e53257ea0d9ae66",
          "message": "Merge pull request #437 from zcash/zip32-arb-keys\n\nMigrate to `zip32::hardened_only` implementation",
          "timestamp": "2025-02-20T17:26:53-07:00",
          "tree_id": "bf069de9e7ae473fb08366c8e6cc4aea79648b1b",
          "url": "https://github.com/zcash/orchard/commit/319d0de29a683af9027bed361e53257ea0d9ae66"
        },
        "date": 1740098387124,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2872221505,
            "range": "± 107567033",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2851024717,
            "range": "± 15979286",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4089054817,
            "range": "± 11693522",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5302586901,
            "range": "± 35749376",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24328975,
            "range": "± 939313",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24385793,
            "range": "± 663032",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 27904477,
            "range": "± 614057",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31359449,
            "range": "± 243039",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1479955,
            "range": "± 11275",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 124976,
            "range": "± 551",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1478315,
            "range": "± 4891",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1316912746,
            "range": "± 7025088",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15643245,
            "range": "± 725643",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2120096,
            "range": "± 225726",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15613240,
            "range": "± 49446",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2085077,
            "range": "± 12430",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78152326,
            "range": "± 100661",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10546466,
            "range": "± 51697",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 78027528,
            "range": "± 203739",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10374274,
            "range": "± 25940",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156326759,
            "range": "± 253402",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21080436,
            "range": "± 148625",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 156015671,
            "range": "± 3031705",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20732048,
            "range": "± 399126",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 462764,
            "range": "± 2748",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488257,
            "range": "± 7939",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "efa62fcf8f03e6be5371d9ddba01a9a4380366bd",
          "message": "Merge pull request #441 from nuttycom/remove_f6_memos\n\nRemove 0xf6 empty memo convention.",
          "timestamp": "2025-02-20T17:30:45-07:00",
          "tree_id": "7ee38c3155a3f430db9db718a4feb2763913de45",
          "url": "https://github.com/zcash/orchard/commit/efa62fcf8f03e6be5371d9ddba01a9a4380366bd"
        },
        "date": 1740098620680,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2870882017,
            "range": "± 245751391",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2845354885,
            "range": "± 4558801",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4073804207,
            "range": "± 11549620",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5309151390,
            "range": "± 39081540",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24305442,
            "range": "± 451940",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24259729,
            "range": "± 1073892",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 27365266,
            "range": "± 253734",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31156547,
            "range": "± 265685",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1482496,
            "range": "± 9936",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125871,
            "range": "± 494",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1478664,
            "range": "± 6293",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1330224549,
            "range": "± 1699346",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15651283,
            "range": "± 34626",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2135559,
            "range": "± 34542",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15635272,
            "range": "± 42496",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2101624,
            "range": "± 7356",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78200594,
            "range": "± 161264",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10632069,
            "range": "± 29242",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 78104608,
            "range": "± 273584",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10450647,
            "range": "± 27399",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156351451,
            "range": "± 175422",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21235633,
            "range": "± 51472",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 156191496,
            "range": "± 900780",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20889469,
            "range": "± 28475",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 463686,
            "range": "± 4324",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 487645,
            "range": "± 718",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}