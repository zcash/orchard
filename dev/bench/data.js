window.BENCHMARK_DATA = {
  "lastUpdate": 1644604040535,
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
          "id": "62afe98f143c58aa242fdcb0908435c0c1c48f0f",
          "message": "Merge pull request #278 from zcash/203-shuffle-spends-and-outputs\n\nShuffle spends and recipients before pairing them into Actions",
          "timestamp": "2022-02-11T11:10:46-07:00",
          "tree_id": "9a5036cf4eee95d1b81ca60bd797884e2ed03d4c",
          "url": "https://github.com/zcash/orchard/commit/62afe98f143c58aa242fdcb0908435c0c1c48f0f"
        },
        "date": 1644604039098,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 3673549907,
            "range": "± 148908499",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 3659194078,
            "range": "± 152735513",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5746321711,
            "range": "± 269196411",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 6811988886,
            "range": "± 11548435",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 31448479,
            "range": "± 317630",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 31485549,
            "range": "± 184737",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 35105275,
            "range": "± 823937",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 35292236,
            "range": "± 400891",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 921802,
            "range": "± 4168",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 118039,
            "range": "± 606",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 920125,
            "range": "± 5222",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 119311782,
            "range": "± 1026643",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 18131449,
            "range": "± 112735",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2074045,
            "range": "± 18373",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 18118736,
            "range": "± 144735",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2040623,
            "range": "± 12560",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 103193366,
            "range": "± 30851",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10271205,
            "range": "± 95455",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 89936016,
            "range": "± 868564",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10170337,
            "range": "± 47381",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 181503748,
            "range": "± 650766",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 23488477,
            "range": "± 83304",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 181419289,
            "range": "± 673300",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20340132,
            "range": "± 92543",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 437857,
            "range": "± 3384",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 480232,
            "range": "± 2324",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}