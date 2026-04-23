const GLTF_EXTENSION = ".gltf";

type UploadUnit =
	| { kind: "single"; id: string; fileName: string; files: File[] }
	| { kind: "bundle"; id: string; fileName: string; files: File[] };

export async function createUploadUnits(
	files: FileList,
): Promise<UploadUnit[]> {
	const droppedFiles = Array.from(files);
	const filesByName = new Map(droppedFiles.map((file) => [file.name, file]));
	const gltfDescriptors = await Promise.all(
		droppedFiles
			.filter((file) => file.name.toLowerCase().endsWith(GLTF_EXTENSION))
			.map(async (file) => ({
				file,
				sidecarNames: await readExternalResourceNames(file),
			})),
	);

	const bundledSidecars = new Set<string>();
	const uploadUnits: UploadUnit[] = [];

	for (const descriptor of gltfDescriptors) {
		const bundledFiles = [descriptor.file];

		for (const sidecarName of descriptor.sidecarNames) {
			const sidecar = filesByName.get(sidecarName);
			if (!sidecar) {
				continue;
			}

			bundledFiles.push(sidecar);
			bundledSidecars.add(sidecar.name);
		}

		uploadUnits.push({
			kind: bundledFiles.length > 1 ? "bundle" : "single",
			id: crypto.randomUUID(),
			fileName: descriptor.file.name,
			files: bundledFiles,
		});
	}

	for (const file of droppedFiles) {
		if (
			file.name.toLowerCase().endsWith(GLTF_EXTENSION) ||
			bundledSidecars.has(file.name)
		) {
			continue;
		}

		uploadUnits.push({
			kind: "single",
			id: crypto.randomUUID(),
			fileName: file.name,
			files: [file],
		});
	}

	return uploadUnits;
}

async function readExternalResourceNames(file: File): Promise<Set<string>> {
	try {
		const gltf = JSON.parse(await file.text()) as {
			buffers?: Array<{ uri?: string }>;
			images?: Array<{ uri?: string }>;
		};
		const names = new Set<string>();

		for (const uri of [
			...(gltf.buffers ?? []).map((buffer) => buffer.uri),
			...(gltf.images ?? []).map((image) => image.uri),
		]) {
			const normalized = normalizeRelativeResourceName(uri);
			if (normalized) {
				names.add(normalized);
			}
		}

		return names;
	} catch {
		return new Set();
	}
}

function normalizeRelativeResourceName(uri: string | undefined): string | null {
	if (!uri || uri.startsWith("data:") || uri.includes(":")) {
		return null;
	}

	const normalizedParts: string[] = [];
	for (const part of uri.split("/")) {
		if (!part || part === ".") {
			continue;
		}
		if (part === "..") {
			if (normalizedParts.length === 0) {
				return null;
			}
			normalizedParts.pop();
			continue;
		}

		normalizedParts.push(part);
	}

	return normalizedParts.length > 0 ? normalizedParts.join("/") : null;
}
