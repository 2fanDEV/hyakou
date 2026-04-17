import { AssetInformation, type Hyako } from "@wasm/hyako_wasm_bindings";
import { type RefObject, useState } from "react";

export default function useFileDrop(hyakou: RefObject<Hyako | null>) {
  const [files, setFiles] = useState<File[]>([]);
  const uploadFile = async (file: File) => {
    if (hyakou.current !== null) {
      setFiles([...files, file]);
      hyakou.current.upload_file(
        new AssetInformation(
          new Uint8Array(await file.arrayBuffer()),
          file.name,
          BigInt(file.size),
          file.lastModified,
        ),
      );
    } else {
      throw new Error("Hyako is not available");
    }
  };
  return { files, uploadFile };
}
