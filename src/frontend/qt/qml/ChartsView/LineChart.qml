import QtQuick

import Theme
import ChartJs 1.0


Item {
    id: root
    property string title: ""
    property int yMax: 100.0
    property int yMin: 0.0
    property string yLabel: "%"

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
                    borderWidth: 2,
                    // pointRadius: 2,
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
                        display: false,
                        type: "time",
                        time: {
                            // Unix timestamp in ms.
                            parser: "x"
                        },
                        scaleLabel: {
                            display: true,
                            // labelString: "Time"
                        },
                        gridLines: {
                            display: false,
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

    function setData(data) {
        chart.chartData.datasets[0].data = data
        chart.animateToNewData()
    }
}