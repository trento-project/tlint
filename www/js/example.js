// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

// Synthetic example check. See the anatomy guide:
// https://www.trento-project.io/docs/wanda/specification.html#_anatomy_of_a_check
export default `id: "0A1B2C"
name: The "example_param" matches the expected value
group: Example
description: Synthetic example check, not a real Trento check.
remediation: Update \`example_param\` in \`example.conf\` to the expected value.
metadata:
  target_type: cluster
  provider:
    - aws
    - azure
    - gcp
facts:
  - name: example_param
    gatherer: example.conf
    argument: sample.value
values:
  - name: expected_example_param
    default: 5000
    conditions:
      - value: 30000
        when: env.provider == "azure" || env.provider == "aws"
      - value: 20000
        when: env.provider == "gcp"
expectations:
  - name: example_param_matches
    expect: facts.example_param == values.expected_example_param
    failure_message: Expected \${values.expected_example_param}, got \${facts.example_param}
`;
