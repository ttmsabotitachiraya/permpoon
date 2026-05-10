import type { RecommendationItem } from "./icode";

export interface PatientInfo {
  hn: string
  fname: string
  lname: string
  cid: string
  pttype: string
  pttype_name: string
  hipdata_code: string
  dob: string
  sex: 'M' | 'F' | string
  age: number
  recommendations: RecommendationItem[];
}

export interface PatientQuery {
  hn?: string;
  cid?: string;
  fname?: string;
  lname?: string;
}
