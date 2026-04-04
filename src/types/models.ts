export type TrackStatus = "Idea" | "Draft" | "In Progress" | "Final";
export type ReleaseType = "Album" | "EP" | "Single";
export type ReleaseStatus = "Planned" | "In Progress" | "Released";

export type Track = {
  id: number;
  internalCode: string;
  title: string;
  status: TrackStatus;
  description: string | null;
  lyrics: string | null;
  notes: string | null;
  bpm: number | null;
  key: string | null;
  createdAt: string;
  updatedAt: string;
};

export type TrackInput = {
  internalCode: string;
  title: string;
  status: TrackStatus;
  description: string | null;
  lyrics: string | null;
  notes: string | null;
  bpm: number | null;
  key: string | null;
};

export type TrackListRow = Track & {
  assignedReleaseId: number | null;
  assignedReleaseTitle: string | null;
  availability: "Available" | "Assigned";
};

export type Release = {
  id: number;
  internalCode: string;
  title: string;
  type: ReleaseType;
  status: ReleaseStatus;
  description: string | null;
  imagePath: string | null;
  trackCount?: number | null;
  createdAt: string;
  updatedAt: string;
};

export type ReleaseInput = {
  internalCode: string;
  title: string;
  type: ReleaseType;
  status: ReleaseStatus;
  description: string | null;
  imagePath: string | null;
};

export type ReleaseTrackRow = {
  trackOrder: number;
  trackId: number;
  internalCode: string;
  title: string;
  status: TrackStatus;
  description: string | null;
  lyrics: string | null;
  bpm: number | null;
  key: string | null;
};

export type DashboardSummary = {
  totalTracks: number;
  availableTracks: number;
  totalReleases: number;
  recentTracks: Track[];
  recentReleases: Release[];
};
