import Decimal from "decimal.js";

export default function Numberify(props: {
  t: { commodityDecimal: number; commodityUnit: string };
  amount: number;
}) {
  let n = Decimal(props.amount);

  if (props.t.commodityDecimal != 0) {
    n = n.div(Math.pow(10, props.t.commodityDecimal));
  }

  let str = n.toFixed(props.t.commodityDecimal);

  if (props.t.commodityUnit) {
    str = str + " " + props.t.commodityUnit;
  }

  return str;
}
