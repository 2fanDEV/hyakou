import type { Icon } from "@phosphor-icons/react";
import {
	ArchiveIcon,
	CubeIcon,
	FileAudioIcon,
	FileCodeIcon,
	FileIcon,
	FileImageIcon,
	FilmStripIcon,
} from "@phosphor-icons/react";

const IMAGE_EXTENSIONS = new Set(["png", "jpg", "jpeg", "gif", "webp", "svg"]);
const AUDIO_EXTENSIONS = new Set(["mp3", "wav", "ogg", "flac"]);
const VIDEO_EXTENSIONS = new Set(["mp4", "mov", "mkv", "webm", "avi"]);
const ARCHIVE_EXTENSIONS = new Set(["zip", "rar", "7z", "tar", "gz"]);
const CODE_EXTENSIONS = new Set([
	"json",
	"js",
	"ts",
	"tsx",
	"jsx",
	"rs",
	"wgsl",
]);
const MODEL_EXTENSIONS = new Set([
	"gltf",
	"glb",
	"obj",
	"mtl",
	"fbx",
	"blend",
	"dae",
	"stl",
]);

function getExtension(fileName: string) {
	const parts = fileName.toLowerCase().split(".");
	return parts.length > 1 ? (parts.at(-1) ?? "") : "";
}

export function getFileIcon(fileName: string): Icon {
	const extension = getExtension(fileName);

	if (IMAGE_EXTENSIONS.has(extension)) return FileImageIcon;
	if (AUDIO_EXTENSIONS.has(extension)) return FileAudioIcon;
	if (VIDEO_EXTENSIONS.has(extension)) return FilmStripIcon;
	if (ARCHIVE_EXTENSIONS.has(extension)) return ArchiveIcon;
	if (MODEL_EXTENSIONS.has(extension)) return CubeIcon;
	if (CODE_EXTENSIONS.has(extension)) return FileCodeIcon;

	return FileIcon;
}
