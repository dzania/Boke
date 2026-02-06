export function ArticleListSkeleton({ count = 5 }: { count?: number }) {
  return (
    <div role="status" aria-label="Loading articles">
      {Array.from({ length: count }, (_, i) => (
        <div
          key={i}
          className="px-4 py-3 border-b animate-pulse"
          style={{ borderColor: "var(--color-border)" }}
        >
          <div className="flex items-start gap-2">
            <div
              className="w-2 h-2 rounded-full mt-1.5 shrink-0"
              style={{ backgroundColor: "var(--color-border)" }}
            />
            <div className="flex-1 min-w-0">
              <div
                className="h-4 rounded"
                style={{
                  backgroundColor: "var(--color-border)",
                  width: `${60 + ((i * 17) % 30)}%`,
                }}
              />
              <div
                className="h-3 rounded mt-1.5"
                style={{
                  backgroundColor: "var(--color-border)",
                  width: `${30 + ((i * 13) % 20)}%`,
                  opacity: 0.6,
                }}
              />
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

export function FeedListSkeleton({ count = 4 }: { count?: number }) {
  return (
    <div role="status" aria-label="Loading feeds">
      {Array.from({ length: count }, (_, i) => (
        <div key={i} className="flex items-center justify-between px-3 py-1.5 mt-0.5 animate-pulse">
          <div
            className="h-4 rounded"
            style={{
              backgroundColor: "var(--color-border)",
              width: `${50 + ((i * 19) % 35)}%`,
            }}
          />
          <div
            className="h-3 w-5 rounded"
            style={{ backgroundColor: "var(--color-border)", opacity: 0.5 }}
          />
        </div>
      ))}
    </div>
  );
}

export function ReaderSkeleton() {
  return (
    <div
      className="py-8 px-6 animate-pulse"
      style={{ maxWidth: 680, margin: "0 auto" }}
      role="status"
      aria-label="Loading article"
    >
      <div
        className="h-7 rounded mb-2"
        style={{ backgroundColor: "var(--color-border)", width: "80%" }}
      />
      <div
        className="h-4 rounded mb-6"
        style={{ backgroundColor: "var(--color-border)", width: "40%", opacity: 0.6 }}
      />
      <div className="space-y-3">
        {[100, 95, 88, 70, 100, 92, 60, 100, 85, 78].map((w, i) => (
          <div
            key={i}
            className="h-4 rounded"
            style={{
              backgroundColor: "var(--color-border)",
              width: `${w}%`,
              opacity: 0.5,
            }}
          />
        ))}
      </div>
    </div>
  );
}
