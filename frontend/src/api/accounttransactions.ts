import { useQuery } from "@tanstack/react-query";
export function useAccountTransactions(accounts_re: string) {
  return useQuery({
    queryKey: ["accounttransactions", accounts_re],
    queryFn: async (): Promise<Ttransactions> => {
      const response = await fetch("/accounttransactions/" + accounts_re);

      return await response.json();
    },
  });
}

export type Ttransactions = Ttransaction[];

export interface Ttransaction {
  tcode: string;
  tcomment: string;
  tdate: string;
  tfulldate: number;
  tdate2: string;
  tfulldate2: number;
  tdescription: string;
  tindex: number;
  tpostings: Tposting[];
  tprecedingcomment: string;
  tsourcepos: any[];
  tstatus: string;
  ttags: any[];
}

export interface Tposting {
  paccount: string;
  pamount: Pamount[];
  pbalanceassertion: any;
  pcomment: string;
  pdate: any;
  pdate2: any;
  poriginal: any;
  pstatus: string;
  ptags: any[];
  ptransaction_: string;
  ptype: string;
}

export interface Pamount {
  acommodity: string;
  acost: any;
  aquantity: Aquantity;
  astyle: Astyle;
}

export interface Aquantity {
  decimalMantissa: number;
  decimalPlaces: number;
  floatingPoint: number;
}

export interface Astyle {
  ascommodityside: string;
  ascommodityspaced: boolean;
  asdecimalmark: string;
  asdigitgroups: any;
  asprecision: number;
  asrounding: string;
}
