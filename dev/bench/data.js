window.BENCHMARK_DATA = {
  "lastUpdate": 1651170933261,
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
          "id": "fbeaff4fd2680929d99ef933e255ce5f0cb99b49",
          "message": "Merge pull request #315 from zcash/msrv-1.56.1-cleanups\n\nCleanups for MSRV 1.56.1",
          "timestamp": "2022-04-28T12:21:04-06:00",
          "tree_id": "e4b3ba0f66925eb4467b5d29b9aab81d57bb9732",
          "url": "https://github.com/zcash/orchard/commit/fbeaff4fd2680929d99ef933e255ce5f0cb99b49"
        },
        "date": 1651170931622,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 5008444777,
            "range": "± 25063241",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 5033013860,
            "range": "± 25911661",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 7214081094,
            "range": "± 30070413",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 9422848636,
            "range": "± 59094332",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 40149118,
            "range": "± 1917751",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 40242123,
            "range": "± 1534970",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 45791922,
            "range": "± 1633850",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 49168096,
            "range": "± 1543231",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}