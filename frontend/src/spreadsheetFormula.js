// import FormulaError from "fast-formula-parser/formulas/error";
import {
  FormulaHelpers,
  // Types,
  // Factorials,
  // Criteria,
} from "fast-formula-parser/formulas/helpers";
const H = FormulaHelpers;

export const customFormulaParserFunctions = {
  SUMIF: (context, range, criteria, sumRange) => {
    const ranges = H.retrieveRanges(context, range, sumRange);

    range = ranges[0];
    sumRange = ranges[1];

    criteria = H.retrieveArg(context, criteria);
    let sum = 0;

    range.forEach((value, rowNum) => {
      const valueToAdd = sumRange[rowNum];

      if (criteria.value === value) {
        if (typeof valueToAdd !== "number") return;

        // wildcard
        sum += valueToAdd;
      }
    });

    return sum;
  },
};
