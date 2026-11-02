# Windows code signing

## Status and requirements

The workflow integration is ready for provider configuration. It does not issue a
certificate or mean that existing downloads are signed. A GitHub personal access
token cannot issue an Authenticode signature. A self-signed certificate is not a
substitute for a certificate trusted by Windows.

[SignPath Foundation](https://signpath.org/) offers free code signing for approved
open-source projects. Apply at <https://signpath.org/apply.html> and read its
[conditions](https://signpath.org/terms.html). Admission is the provider's decision;
this repository is a fork, so discuss the upstream/fork eligibility requirements
with the Foundation. No provider acceptance or sponsorship is claimed yet.

## Code signing policy

- Repository owner, maintainer, reviewer, and signing approver:
  [Kerim Demirkaynak](https://github.com/KerimDemirkaynak).
- Signing is an explicit manual release operation from this repository's default
  branch. Pull-request builds and normal development builds are unsigned.
- Configure the SignPath policy to require the maintainer's approval and verify
  the source repository and default branch. Follow the provider's review and MFA
  requirements. Signing credentials are used only by the signing steps.
- Only `assdraw3.exe` and the outer Inno Setup installer are signed by this
  integration. Third-party runtime DLLs retain their original signatures. Inno
  Setup's generated uninstaller is not separately signed by this integration.
- The application has no background telemetry. User-requested external links open
  in the system browser. The signing service receives build artifacts and their
  GitHub provenance, not application users' drawings or settings. See
  [SignPath's privacy policy](https://signpath.io/privacy-policy) for the service.

After Foundation approval, add the required provider attribution to the project
home page and release/download pages:

> Free code signing provided by SignPath.io, certificate by SignPath Foundation

Do not present that statement as an existing sponsorship before approval.

## One-time provider setup

1. Obtain project approval and the signing certificate. Install the
   [SignPath GitHub App](https://github.com/apps/signpath) for this repository and
   connect the project to GitHub as a trusted build system.
2. In the SignPath project, create these two artifact configurations from the
   checked-in XML files:

   | Configuration slug | XML file |
   | --- | --- |
   | `windows-application` | [windows-application.xml](../build/signpath/windows-application.xml) |
   | `windows-installer` | [windows-installer.xml](../build/signpath/windows-installer.xml) |

   Both receive the ZIP envelope created by `actions/upload-artifact`. Keep that
   upload's default archive mode. Only the exact application/installer file is
   selected, with restrictions on the product name and product version. Update
   the XML restrictions together with `src/assdraw.rc` and `setup.iss` when the
   product version changes.
3. Create a release signing policy with the trusted production certificate,
   Authenticode timestamping, a CI submitter, and the maintainer as approver.
   Enforce this repository, its default branch, and GitHub-hosted build agents in
   the provider's pipeline policy.
4. In GitHub **Settings → Secrets and variables → Actions**, add:

   | Type | Name | Value |
   | --- | --- | --- |
   | Secret | `SIGNPATH_API_TOKEN` | CI submitter's SignPath API token |
   | Variable | `SIGNPATH_ORGANIZATION_ID` | SignPath organization ID |
   | Variable | `SIGNPATH_PROJECT_SLUG` | SignPath project slug |
   | Variable | `SIGNPATH_SIGNING_POLICY_SLUG` | Approved release policy slug |

   This is a SignPath token, not the GitHub token. Keep it in GitHub Secrets.
   Do not commit a private key, a PFX file, or a token to the repository.

## Produce signed packages

Open **Actions → Build ASSDraw3 → Run workflow**, select the default branch, choose
the desired Windows format, and enable **sign_windows**. Complete the provider's
approval requests while the workflow waits (up to 30 minutes per request).

The workflow:

1. Rejects signing requests from any other repository/branch and checks that all
   provider settings are present before building.
2. Builds and tests the application before requesting its signature.
3. Uploads the unsigned application as a short-lived `signing-input-application`
   artifact, requests its signature, and verifies its embedded signature against
   Windows trust plus the presence of a timestamp.
4. Builds the portable package and installer from that signed application. The
   application is never rebuilt after signing.
5. If selected, uploads `signing-input-installer`, signs the installer, and checks
   its signature, timestamp, and signing certificate against the application.
6. Installs the final installer in a temporary directory and checks that its
   application is byte-for-byte identical to the signed application.
7. Publishes the selected final packages only after all applicable checks pass.

A signing or verification failure stops the job; there is no unsigned fallback
for a requested signed build. The workflow summary explicitly states whether the
build was signed. The `signing-input-*` artifacts are unsigned inputs for the
service, expire after one day, and are not release downloads.

Local builds and builds with **sign_windows** disabled require no signing
account and remain unsigned. The signing steps cannot be tested against a real
production certificate until provider enrollment is complete. CI does test the
verification gate with a genuine Microsoft-signed executable, an unsigned
application, a missing file, and a tampered signed executable.

## Verify a download

In PowerShell on Windows, run:

```powershell
Get-AuthenticodeSignature .\ASSDraw3-Windows-x64-Setup.exe |
    Format-List Status, StatusMessage, SignerCertificate, TimeStamperCertificate
```

The signature must be `Valid` and the publisher must match the approved signing
provider/certificate. The same check applies to `assdraw3.exe` inside the portable
or installed package. Repository maintainers can use
`build/verify-windows-signature.ps1` for the strict check used by CI.

Code signing authenticates the publisher and detects modifications. It does not
prove that software is free of malware or guarantee that Chrome Safe Browsing or
Windows SmartScreen will stop displaying warnings. Investigate any remaining
malware detections separately using the exact affected binary.

References: [SignPath GitHub integration](https://docs.signpath.io/trusted-build-systems/github),
[artifact configurations](https://docs.signpath.io/artifact-configuration/),
[Google download guidelines](https://developers.google.com/search/docs/monitor-debug/security/malware),
[Microsoft app reputation](https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation).
