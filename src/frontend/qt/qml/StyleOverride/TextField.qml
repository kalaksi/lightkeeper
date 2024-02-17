import QtQuick 2.15
import QtQuick.Controls 2.15
import org.kde.kirigami 2.19 as Kirigami


TextField {
    Kirigami.Theme.inherit: false
    Kirigami.Theme.textColor: Theme.textColor
    Kirigami.Theme.disabledTextColor: Theme.disabledTextColor
}