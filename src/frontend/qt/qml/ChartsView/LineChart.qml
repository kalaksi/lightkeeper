import QtQuick

import Theme
import ChartJs 1.0


Item {
    id: root
    property string title: ""
    property int yMax: 100.0
    property int yMin: 0.0
    property string yLabel: "%"
    property var chartData: []

    function setData(data) {
        // console.log("Chart '" + root.title + "' data set: " + JSON.stringify(data))
        if (!data) {
            chart.chartData.datasets[0].data = []
            return
        }

        // Convert "t" to "x" for Chart.js time series format. Used to work, doesn't anymore.
        // TODO: should have "x" to begin with.
        var convertedData = data.map(function(item) {
            if (item.t !== undefined) {
                return {"x": item.t, "y": item.y}
            }
            return item
        })
        chart.chartData.datasets[0].data = convertedData

        // Sometimes the chart instance hasn't been created yet.
        if (chart.jsChart !== undefined) {
            chart.animateToNewData()
        }
    }

    Chart {
        id: chart
        anchors.fill: parent
        chartType: "line"
        chartData: {
            return {
                datasets: [{
                    label: "",
                    fill: true,
                    backgroundColor: "rgba(100,200,100,0.3)",
                    borderColor: "rgba(255,255,255,1.0)",
                    borderWidth: 1,
                    pointRadius: 1,
                    data: [],
                }]
            }
        }

        chartOptions: {
            return {
                maintainAspectRatio: false,
                responsive: true,
                title: {
                    display: true,
                    text: root.title,
                    fontColor: Theme.textColor,
                    padding: 5,
                    lineHeight: 1.0
                },
                tooltips: {
                    mode: "index",
                    intersect: false,
                },
                hover: {
                    mode: "nearest",
                    intersect: true
                },
                legend: {
                    display: false,
                    labels: {
                        fontColor: Theme.textColor
                    }
                },
                scales: {
                    xAxes: [{
                        display: true,
                        type: "time",
                        time: {
                            // Unix timestamp in ms.
                            parser: "x",
                            unit: "day",
                            displayFormats: {
                                day: "DD"
                            }
                        },
                        scaleLabel: {
                            display: true,
                            // labelString: "Time"
                        },
                        gridLines: {
                            display: true,
                            color: "rgba(255,255,255,0.1)"
                        },
                        ticks: {
                            maxTicksLimit: 12,
                            fontColor: Theme.textColor,
                            // Performance optimization:
                            maxRotation: 0,
                            minRotation: 0,
                        }
                    }],
                    yAxes: [{
                        display: true,
                        scaleLabel: {
                            display: true,
                            labelString: root.yLabel,
                            fontColor: Theme.textColor
                        },
                        gridLines: {
                            display: true,
                            color: "rgba(255,255,255,0.1)"
                        },
                        ticks: {
                            min: root.yMin,
                            max: root.yMax,
                            maxTicksLimit: 8,
                            fontColor: Theme.textColor,
                            // Performance optimization:
                            maxRotation: 0,
                            minRotation: 0,
                        }
                    }]
                }
            }
        }
    }
}