export function suggestedExportName(title: string): string {
  const cleaned = title.replace(/[^\p{L}\p{N} -]/gu, "_").trim();
  return `${cleaned || "note"}.md`;
}
