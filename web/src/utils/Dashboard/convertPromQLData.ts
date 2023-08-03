import { onMounted, reactive, ref } from "vue";
import Plotly from "plotly.js";
import { getThemeLayoutOptions } from "@/utils/Dashboard/getThemeLayoutOptions";
import moment from "moment";

export const convertPromQLData = (
  panelSchema: any,
  searchQueryDataTemp: any,
  store: any
) => {
  const props = {
    data: panelSchema,
  };

  const searchQueryData = {
    data: searchQueryDataTemp,
  };
  const getPromqlLegendName = (metric: any, label: string) => {
    if (label) {
      let template = label || "";
      const placeholders = template.match(/\{([^}]+)\}/g);

      // Step 2: Iterate through each placeholder
      placeholders?.forEach(function (placeholder: any) {
        // Step 3: Extract the key from the placeholder
        const key = placeholder.replace("{", "").replace("}", "");

        // Step 4: Retrieve the corresponding value from the JSON object
        const value = metric[key];

        // Step 5: Replace the placeholder with the value in the template
        if (value) {
          template = template.replace(placeholder, value);
        }
      });
      return template;
    } else {
      return JSON.stringify(metric);
    }
  };

  const getLegendPosition = (type: string) => {
    const legendPosition = props.data.config?.legends_position;

    switch (legendPosition) {
      case "bottom":
        return "h";
      case "right":
        return "v";
      default:
        return type == "promql" ? "h" : "v";
    }
  };

  const getUnitValue = (value: any) => {
    switch (props.data.config?.unit) {
      case "bytes": {
        const units = ["B", "KB", "MB", "GB", "TB"];
          for (let unit of units) {
            if (value < 1024) {
                return {
                    value: `${parseFloat(value).toFixed(2)}`,
                    unit: `${unit}`
                }
            }
            value /= 1024;
          }
          return {
            value:`${parseFloat(value).toFixed(2)}`,
            unit: 'PB'
          };
      }
      case "custom": {
          return {
            value: `${value}`,
            unit: `${props.data.config.unit_custom ?? ''}`
          }
      }
      case "seconds": {
        const units = [
            { unit: "ms", divisor: 0.001 },
            { unit: "s", divisor: 1 },
            { unit: "m", divisor: 60 },
            { unit: "h", divisor: 3600 },
            { unit: "D", divisor: 86400 },
            { unit: "M", divisor: 2592000 }, // Assuming 30 days in a month
            { unit: "Y", divisor: 31536000 }, // Assuming 365 days in a year
        ];
        for (const unitInfo of units) {
            const unitValue = value ? value / unitInfo.divisor : 0 ;
            if (unitValue >= 1 && unitValue < 1000) {
                return {
                    value: unitValue.toFixed(2),
                    unit: unitInfo.unit
                }
            }
        }

        // If the value is too large to fit in any unit, return the original seconds
        return {
            value: value,
            unit: 's'
        }
      }
      case "bps": {
        const units = ["B", "KB", "MB", "GB", "TB"];
        for (let unit of units) {
          if (value < 1024) {
              return {
                  value: `${parseFloat(value).toFixed(2)}`,
                  unit: `${unit}/s`
              }
          }
          value /= 1024;
        }
        return {
          value:`${parseFloat(value).toFixed(2)}`,
          unit: 'PB/s'
        }
      }
      case "percent-1":{
        return {
          value: `${(parseFloat(value)  * 100).toFixed(2)}`,
          unit: '%'
        }
        // `${parseFloat(value) * 100}`;
      }
      case "percent":{
        return {
          value: `${parseFloat(value).toFixed(2)}`,
          unit: '%'
        }
        // ${parseFloat(value)}`;
      }
      case "default":{
        return {
          value: value,
          unit: ''
        }
      }
      default:{
        return {
          value: value,
          unit: ''
        } 
      }
    }
  }

  const formatUnitValue = (obj:any) => {
    return `${obj.value}${obj.unit}`
  }

  const getPropsByChartTypeForTraces = () => {
    switch (props.data.type) {
      case "bar":
        return {
          type: "bar",
        };
      case "line":
        return {
          mode: "lines",
        };
      case "scatter":
        return {
          mode: "markers",
        };
      case "pie":
        return {
          type: "pie",
        };
      case "donut":
        return {
          type: "pie",
        };
      case "h-bar":
        return {
          type: "bar",
          orientation: "h",
        };
      case "area":
        return {
          fill: "tozeroy", //TODO: hoe to change the color of plot chart
          type: "scatter",
        };
      case "stacked":
        return {
          type: 'bar',
        };
      case "area-stacked":
        return {
          mode: 'lines',  
          // fill: 'none'
        };
      case "metric":
        return {
          type: "indicator",
          mode: "number"
        };
      case "h-stacked":
        return {
          type: 'bar',
          orientation: "h",
        };
      default:
        return {
          type: "bar",
        };
    }
  };

  const getPropsByChartTypeForLayoutForPromQL = () => {
      //   console.log("data with tick",xAxisDataWithTicks);
    switch (props.data.type) {
      case "bar": {
        const trace = {
          barmode: "group",
        }
        return trace
      }

      case "line": {
        return
      }

      case "scatter": {
        const trace = {
          scattermode: "group",
        }
        return trace
      }

      case "pie":{
        return
      }
          
      case "donut":{
        return
      }
          
      case "h-bar": {
        const trace = {
          barmode: "group",
        }
        return trace
      }

      case "area": {
        return
      }
        
      case "area-stacked": {
        const layout = {
          barmode: "stack",
        }
        return layout
      }

      case "stacked": {
        const layout = {
          barmode: "stack",
        }
        return layout
      }

      case "h-stacked":{
        return
      }
          
      case "metric":{
        return
      }
    }
  };
  console.log("props", props);
  console.log("convertPromQLData: searchQueryData", searchQueryDataTemp);

  const traces = searchQueryData.data.map((it: any, index: number) => {
    switch (props.data.type) {
      case "bar":
      case "line":
      case "area":
      case "scatter":
      case "area-stacked": {
        switch (it.resultType) {
          case "matrix": {
            const traces = it?.result?.map((metric: any) => {
              const values = metric.values.sort(
                (a: any, b: any) => a[0] - b[0]
              );
              return {
                name: getPromqlLegendName(
                  metric.metric,
                  props.data.queries[index].promql_legend
                ),
                x: values.map((value: any) =>
                  moment(value[0] * 1000).toISOString(true)
                ),
                y: values.map((value: any) => value[1]),
                hovertext: values.map((value: any) =>
                  formatUnitValue(getUnitValue(value[1]))
                ),
                hovertemplate:
                  "%{x} <br>%{fullData.name}: %{hovertext}<extra></extra>",
                stackgroup: props.data.type == "area-stacked" ? "one" : "",
                ...getPropsByChartTypeForTraces(),
              };
            });
            // Calculate the maximum value size from the 'y' values in the 'traces' array
            const maxValueSize =
              props.data.type == "area-stacked"
                ? traces.reduce(
                    (sum: any, it: any) => sum + Math.max(...it.y),
                    0
                  )
                : traces.reduce(
                    (max: any, it: any) => Math.max(max, Math.max(...it.y)),
                    0
                  );

            // Calculate the minimum value size from the 'y' values in the 'traces' array
            const minValueSize = traces.reduce(
              (min: any, it: any) => Math.min(min, Math.min(...it.y)),
              Infinity
            );

            // Initialize empty arrays to hold tick values and tick text
            let yTickVals = [];
            let yTickText = [];

            // Calculate the interval size for 5 equally spaced ticks
            let intervalSize = (maxValueSize - minValueSize) / 4;

            // If the data doesn't vary much, use a percentage of the max value as the interval size
            if (intervalSize === 0) {
              intervalSize = maxValueSize * 0.2;
            }

            // Generate tick values and tick text for the y-axis
            for (let i = 0; i <= 4; i++) {
              let val = minValueSize + intervalSize * i;
              yTickVals.push(minValueSize + intervalSize * i);
              yTickText.push(formatUnitValue(getUnitValue(val)));
            }
            // result = result.map((it: any) => moment(it + "Z").toISOString(true))
            const yAxisTickOptions = !props.data.config?.unit
              ? {}
              : { tickvals: yTickVals, ticktext: yTickText };

            const layout: any = {
              title: false,
              showlegend: props.data.config?.show_legends,
              autosize: true,
              legend: {
                // bgcolor: "#f7f7f7",
                orientation: getLegendPosition("promql"),
                itemclick: false,
              },
              margin: {
                autoexpand: true,
                r: 50,
                b: 50,
                t: 30,
              },
              yaxis: {
                automargin: true,
                autorange: true,
                ...yAxisTickOptions,
              },
              ...getPropsByChartTypeForLayoutForPromQL(),
              ...getThemeLayoutOptions(store),
            };
            console.log("layout", layout);
              // return { traces: traces.flat(), layout };
          }
          case "vector": {
            const traces = it?.result?.map((metric: any) => {
              const values = [metric.value];
              // console.log('vector',values);

              return {
                name: JSON.stringify(metric.metric),
                x: values.map((value: any) =>
                  moment(value[0] * 1000).toISOString(true)
                ),
                y: values.map((value: any) => value[1]),
              };
            });
            const layout: any = {
              title: false,
              showlegend: props.data.config?.show_legends,
              autosize: true,
              legend: {
                // bgcolor: "#f7f7f7",
                orientation: getLegendPosition("promql"),
                itemclick: false,
              },
              margin: {
                l: props.data.type == "pie" ? 60 : 32,
                r: props.data.type == "pie" ? 60 : 16,
                t: 38,
                b: 32,
              },
              ...getThemeLayoutOptions(store),
            };
            return { traces: traces.flat(), layout };
            break;
          }
        }
      }
      case "metric": {
        switch (it.resultType) {
          case "matrix": {
            const traces = it?.result?.map((metric: any) => {
              const values = metric.values.sort(
                (a: any, b: any) => a[0] - b[0]
              );
              const unitValue = getUnitValue(values[values.length - 1][1]);

              return {
                value: unitValue.value,
                number: { suffix: unitValue.unit, valueformat: ".2f" },
                ...getPropsByChartTypeForTraces(),
              };
            });

            // result = result.map((it: any) => moment(it + "Z").toISOString(true))

            const layout: any = {
              title: false,
              showlegend: props.data.config?.show_legends,
              autosize: true,
              legend: {
                // bgcolor: "#f7f7f7",
                orientation: getLegendPosition("promql"),
                itemclick: false,
              },
              margin: {
                autoexpand: true,
                l: 10,
                r: 10,
                t: 0,
                b: 0,
              },
              ...getPropsByChartTypeForLayoutForPromQL(),
              ...getThemeLayoutOptions(store),
            };
            break;
          }
          case "vector": {
            const traces = it?.result?.map((metric: any) => {
              const values = [metric.value];

              // console.log('vector',values);

              return {
                name: JSON.stringify(metric.metric),
                value: metric?.value?.length > 1 ? metric.value[1] : "",
                ...getPropsByChartTypeForTraces(),
              };
            });

            const layout: any = {
              title: false,
              showlegend: props.data.config?.show_legends,
              autosize: true,
              legend: {
                // bgcolor: "#f7f7f7",
                orientation: getLegendPosition("promql"),
                itemclick: false,
              },
              margin: {
                l: props.data.type == "pie" ? 60 : 32,
                r: props.data.type == "pie" ? 60 : 16,
                t: 38,
                b: 32,
              },
              ...getPropsByChartTypeForLayoutForPromQL(),
              ...getThemeLayoutOptions(store),
            };
            break;
          }
        }
        break;
      }
      default: {
        return [];
      }
    }
  });
}