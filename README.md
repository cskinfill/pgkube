A small project to play around with Rust and building a kubernetes operator.

The idea for this project is to find `integration.service.pagerduty.cs.dev` resources in a kubernetes cluster, create a corresponding _Integration_ in the _PagerDuty Service_ referenced in the resource, and update a kubernetes _Secret_ with the Integration's key.  This would allow the prometheus operator (or victoriametrics operator) to use that _Secret_ in the AlertManager configuration.
