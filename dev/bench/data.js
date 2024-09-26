window.BENCHMARK_DATA = {
  "lastUpdate": 1727390949217,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "85249b4fcbebfdaa105e03d357db82f5d67c52c3",
          "message": "Merge pull request #436 from zcash/incrementalmerkletree_0.7\n\nMigrate to incrementalmerkletree version 0.7",
          "timestamp": "2024-09-26T23:35:18+01:00",
          "tree_id": "8f1ce0ac68157c7d0bdb15d28961d69c91fe4121",
          "url": "https://github.com/zcash/orchard/commit/85249b4fcbebfdaa105e03d357db82f5d67c52c3"
        },
        "date": 1727390948269,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2901470425,
            "range": "± 23435514",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2899613350,
            "range": "± 18010482",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4172835136,
            "range": "± 39877276",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5442619458,
            "range": "± 31492807",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24871564,
            "range": "± 205495",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24895682,
            "range": "± 236311",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28267622,
            "range": "± 306505",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31838848,
            "range": "± 224668",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1536942,
            "range": "± 15327",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 128470,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1533300,
            "range": "± 12189",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1379791220,
            "range": "± 1845116",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16212870,
            "range": "± 36545",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2189328,
            "range": "± 10501",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16186391,
            "range": "± 31133",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2145257,
            "range": "± 7066",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80982075,
            "range": "± 89880",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10892140,
            "range": "± 70551",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80890063,
            "range": "± 149397",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10671984,
            "range": "± 37919",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 162011984,
            "range": "± 1202675",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21766877,
            "range": "± 47122",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161786900,
            "range": "± 350037",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21328441,
            "range": "± 57773",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 470284,
            "range": "± 4987",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 505931,
            "range": "± 911",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}