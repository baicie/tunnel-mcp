export type UpdateCheckResult = {
  available: boolean;
  currentVersion: string;
  latestVersion?: string;
  notes?: string;
};

export type TunnelClientVersionStatus = {
  installed: boolean;
  currentVersion?: string;
  latestVersion?: string;
  updateAvailable: boolean;
  assetUrl?: string;
  assetSha256?: string;
  checksumVerified: boolean;
};
