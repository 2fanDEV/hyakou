import { Collapsible, CollapsibleTrigger } from "#/components/ui/collapsible";

type FilesProps = {
  files: File[];
};

export default function FileDirectory({ files }: FilesProps) {
  return (
    <div className="absolute z-10">
      <Collapsible>
        <CollapsibleTrigger> a </CollapsibleTrigger>
        {files.map((file) => (
          <h3 key={file.name}>{file.name}</h3>
        ))}
      </Collapsible>
    </div>
  );
}
