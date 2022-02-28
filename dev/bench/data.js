window.BENCHMARK_DATA = {
  "lastUpdate": 1646084156209,
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
          "id": "3ddf6c49f7484ed1295bd5351317bbfe49e14472",
          "message": "Merge pull request #293 from zcash/merge-non-consensus-changes-2\n\nMerge non-consensus changes again",
          "timestamp": "2022-02-28T21:17:26Z",
          "tree_id": "cfebc5dadd5f46d579914daad36e9fd8d69a1101",
          "url": "https://github.com/zcash/orchard/commit/3ddf6c49f7484ed1295bd5351317bbfe49e14472"
        },
        "date": 1646084154751,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4632576046,
            "range": "± 88827483",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4529242814,
            "range": "± 73759223",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 6621767750,
            "range": "± 82859844",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 8582050016,
            "range": "± 194066343",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 35145682,
            "range": "± 2010362",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 35410467,
            "range": "± 2187548",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 39616917,
            "range": "± 2561778",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 42892919,
            "range": "± 2060652",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1354772,
            "range": "± 50780",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 172459,
            "range": "± 10676",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1073631,
            "range": "± 72405",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 140677043,
            "range": "± 7523016",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 26606263,
            "range": "± 1029498",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2806189,
            "range": "± 137765",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 20915446,
            "range": "± 1184542",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2406622,
            "range": "± 169998",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 129797338,
            "range": "± 3296114",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 12658176,
            "range": "± 781861",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 104842934,
            "range": "± 5723049",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 12077427,
            "range": "± 734709",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 259917784,
            "range": "± 6213563",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 25071117,
            "range": "± 1672829",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 220946964,
            "range": "± 16397179",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 24422000,
            "range": "± 1408076",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 527938,
            "range": "± 29019",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 573595,
            "range": "± 60873",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}