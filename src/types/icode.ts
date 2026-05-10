export type FreqType =
  | "days"
  | "months"
  | "years"
  | "per_week"
  | "per_year"
  | "total_limit"
  | null;

export interface IcodeConfig {
  id: number;
  icode: string;
  service_name: string;
  is_enabled: boolean;
  age_min: number | null;
  age_max: number | null;
  gender_restrict: "M" | "F" | null;
  freq_type: FreqType;
  freq_value: number | null;
  pttype_group_ids: number[];
  // optional department/clinic name where the service is provided
  department?: string | null;
}

export interface RecommendationItem {
  icode: string;
  service_name: string;
  department?: string | null;
  pttype_alias: string[];
}
