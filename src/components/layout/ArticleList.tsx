export default function ArticleList() {
  return (
    <div
      className="flex flex-col items-center justify-center py-12 text-center px-4"
      style={{ color: "var(--color-text-muted)" }}
    >
      <p className="text-sm">No articles yet</p>
      <p className="text-xs mt-1">Add a feed to see articles here</p>
    </div>
  );
}
