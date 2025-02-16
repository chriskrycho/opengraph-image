A small Rust application to generate social media images (for OpenGraph etc.).

Here is the basic design:

1. It will run in a serverless environment that supports running Rust natively.

2. It will be deployed at `og.chriskrycho.com`.

3. It will always generate images at 1.91:1 ratio, 800px high and 1520px wide.

4. It will only respond to requests from subdomains of chriskrycho.com; all subdomains of chriskrycho.com are allowed. Anything else will return a 403 HTTP response.

5. It will cache requests so that a previously-generated image is used again, rather than regenerated each time.

6. It will incorporate the fonts directly into the binary using the `include_bytes!` library:

	| Font file path | Resulting font name |
	| -------------- | ------------------- |
	| `fonts/SanomatSansText-SemiboldItalic.otf` | `SANOMAT_SANS_TEXT_SEMIBOLD_ITALIC` |
	| `fonts/Sanomat-Semibold.otf` | `SANOMAT_SEMIBOLD` |
	| `fonts/FrameHead.otf` | `FRAME_HEAD` |
	| `fonts/FrameHead-Italic.otf` | `FRAME_HEAD_ITALIC` |

7. It will accept a URL-encoded string in a `title` query param, like `og.chriskrycho.com/?title=How%20Progress%20Feels`. From this, it will produce an image of the specified dimensions with the URL-encoded text decoded into a normal string, like `How Progress Feels`.

8. Assets will be deployed to Backblaze B2.
