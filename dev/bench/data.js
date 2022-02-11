window.BENCHMARK_DATA = {
  "lastUpdate": 1644617082929,
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
          "id": "b7f66b48e6051666056563710a29293191d4c4a2",
          "message": "Merge pull request #280 from nuttycom/decrypt_diversifier\n\nAdd diversifier index decryption to DiversifierKey",
          "timestamp": "2022-02-11T14:51:54-07:00",
          "tree_id": "fcd4190951401c4759a330d33328b701468a7669",
          "url": "https://github.com/zcash/orchard/commit/b7f66b48e6051666056563710a29293191d4c4a2"
        },
        "date": 1644617081976,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4157303657,
            "range": "± 14921262",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4142069275,
            "range": "± 15311754",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5935845445,
            "range": "± 7587442",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7724214703,
            "range": "± 17041173",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 32509215,
            "range": "± 440336",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 32251784,
            "range": "± 225486",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 36277942,
            "range": "± 244130",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 40105370,
            "range": "± 301246",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}