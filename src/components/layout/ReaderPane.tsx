export default function ReaderPane() {
  return (
    <main
      className="flex-1 h-full overflow-y-auto"
      style={{ backgroundColor: "var(--color-bg-primary)" }}
    >
      <div
        className="flex flex-col items-center justify-center h-full text-center px-4"
        style={{ color: "var(--color-text-muted)" }}
      >
        <p className="text-lg font-medium">No article selected</p>
        <p className="text-sm mt-1">Select an article from the sidebar to start reading</p>
      </div>
    </main>
  );
}
