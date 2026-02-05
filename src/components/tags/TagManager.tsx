import { useState } from "react";
import { useTags, useCreateTag, useTagFeed, useUntagFeed, useDeleteTag } from "../../api/tags";
import TagBadge from "./TagBadge";
import type { FeedWithMeta } from "../../types";

interface TagManagerProps {
  feed: FeedWithMeta;
  open: boolean;
  onClose: () => void;
}

export default function TagManager({ feed, open, onClose }: TagManagerProps) {
  const [newTagName, setNewTagName] = useState("");
  const { data: tags } = useTags();
  const createTag = useCreateTag();
  const tagFeed = useTagFeed();
  const untagFeed = useUntagFeed();
  const deleteTag = useDeleteTag();

  if (!open) return null;

  const feedTagIds = new Set(
    tags?.filter((t) => t.feed_ids.includes(feed.id)).map((t) => t.id) ?? []
  );

  const handleCreateAndAssign = (e: React.FormEvent) => {
    e.preventDefault();
    const name = newTagName.trim();
    if (!name) return;
    createTag.mutate(name, {
      onSuccess: (tag) => {
        tagFeed.mutate({ feedId: feed.id, tagId: tag.id });
        setNewTagName("");
      },
    });
  };

  const handleToggleTag = (tagId: number) => {
    if (feedTagIds.has(tagId)) {
      untagFeed.mutate({ feedId: feed.id, tagId });
    } else {
      tagFeed.mutate({ feedId: feed.id, tagId });
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ backgroundColor: "rgba(0,0,0,0.5)" }}
      onClick={onClose}
    >
      <div
        className="rounded-lg p-6 w-[380px] shadow-xl"
        style={{ backgroundColor: "var(--color-bg-primary)", border: "1px solid var(--color-border)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-lg font-semibold mb-1" style={{ color: "var(--color-text-primary)" }}>
          Tags for {feed.title}
        </h2>
        <p className="text-xs mb-4" style={{ color: "var(--color-text-muted)" }}>
          Click a tag to toggle assignment
        </p>

        {tags && tags.length > 0 && (
          <div className="flex flex-wrap gap-1.5 mb-4">
            {tags.map((tag) => (
              <span
                key={tag.id}
                className="inline-flex items-center gap-1"
              >
                <TagBadge
                  name={tag.name}
                  onClick={() => handleToggleTag(tag.id)}
                />
                {feedTagIds.has(tag.id) && (
                  <span className="text-xs" style={{ color: "var(--color-accent)" }}>✓</span>
                )}
              </span>
            ))}
          </div>
        )}

        <form onSubmit={handleCreateAndAssign} className="flex gap-2 mb-4">
          <input
            type="text"
            value={newTagName}
            onChange={(e) => setNewTagName(e.target.value)}
            placeholder="New tag name"
            className="flex-1 px-3 py-1.5 rounded-md text-sm outline-none"
            style={{
              backgroundColor: "var(--color-bg-secondary)",
              color: "var(--color-text-primary)",
              border: "1px solid var(--color-border)",
            }}
          />
          <button
            type="submit"
            disabled={!newTagName.trim() || createTag.isPending}
            className="px-3 py-1.5 rounded-md text-sm font-medium text-white disabled:opacity-50"
            style={{ backgroundColor: "var(--color-accent)" }}
          >
            Add
          </button>
        </form>

        {tags && tags.length > 0 && (
          <div className="border-t pt-3" style={{ borderColor: "var(--color-border)" }}>
            <p className="text-xs mb-2" style={{ color: "var(--color-text-muted)" }}>
              Delete tags
            </p>
            <div className="flex flex-wrap gap-1.5">
              {tags.map((tag) => (
                <TagBadge
                  key={tag.id}
                  name={tag.name}
                  onRemove={() => deleteTag.mutate(tag.id)}
                />
              ))}
            </div>
          </div>
        )}

        <div className="flex justify-end mt-4">
          <button
            type="button"
            onClick={onClose}
            className="px-3 py-1.5 rounded-md text-sm"
            style={{ color: "var(--color-text-secondary)" }}
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
}
