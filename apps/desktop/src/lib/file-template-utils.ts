export interface TemplateFileInfo {
    title: string;
    fileName: string;
}

export function templateInfoFromPath(path: string): TemplateFileInfo {
    const fileName = path.split('/').filter(Boolean).at(-1) ?? path;
    const extensionIndex = fileName.lastIndexOf('.');
    const title = extensionIndex > 0 ? fileName.slice(0, extensionIndex) : fileName;

    return { title, fileName };
}
