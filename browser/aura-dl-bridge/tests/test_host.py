import importlib.util
import io
import json
import struct
import unittest
from importlib.machinery import SourceFileLoader
from pathlib import Path

HOST_PATH = Path(__file__).resolve().parents[1] / "aura-dl-host"
LOADER = SourceFileLoader("aura_dl_host", str(HOST_PATH))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
assert SPEC is not None
HOST = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(HOST)


class ValidateTests(unittest.TestCase):
    def test_accepts_valid_video_request(self):
        options, error = HOST.validate(
            {
                "url": "https://example.com/video",
                "mode": "video",
                "container": "mp4",
                "quality": 1080,
            }
        )

        self.assertIsNone(error)
        self.assertEqual(options["url"], "https://example.com/video")
        self.assertEqual(options["quality"], 1080)

    def test_rejects_non_http_url(self):
        options, error = HOST.validate({"url": "file:///etc/passwd"})

        self.assertIsNone(options)
        self.assertIn("http", error)

    def test_rejects_credentials_in_url(self):
        options, error = HOST.validate({"url": "https://user:pass@example.com/video"})

        self.assertIsNone(options)
        self.assertIn("credenciais", error)

    def test_rejects_invalid_quality_without_raising(self):
        options, error = HOST.validate(
            {"url": "https://example.com/video", "quality": "infinite"}
        )

        self.assertIsNone(options)
        self.assertIn("qualidade", error)

    def test_rejects_boolean_quality(self):
        options, error = HOST.validate(
            {"url": "https://example.com/video", "quality": True}
        )

        self.assertIsNone(options)
        self.assertIn("qualidade", error)


class ManifestTests(unittest.TestCase):
    def test_native_host_uses_system_binary_path(self):
        manifest_path = HOST_PATH.parent / "aura_dl.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

        self.assertEqual(manifest["path"], "/usr/bin/aura-dl-host")
        self.assertEqual(manifest["allowed_extensions"], ["aura-dl@aura-os.local"])


class ProtocolTests(unittest.TestCase):
    def test_reads_length_prefixed_json(self):
        payload = json.dumps({"url": "https://example.com/video"}).encode("utf-8")
        stream = io.BytesIO(struct.pack("<I", len(payload)) + payload)

        message = HOST.read_message(stream)

        self.assertEqual(message["url"], "https://example.com/video")

    def test_rejects_non_object_message(self):
        payload = json.dumps(["not", "an", "object"]).encode("utf-8")
        stream = io.BytesIO(struct.pack("<I", len(payload)) + payload)

        with self.assertRaises(ValueError):
            HOST.read_message(stream)

    def test_rejects_oversized_message(self):
        stream = io.BytesIO(struct.pack("<I", HOST.MAX_MESSAGE_BYTES + 1))

        with self.assertRaises(ValueError):
            HOST.read_message(stream)


if __name__ == "__main__":
    unittest.main()
