#pragma once

#include <QQuickItem>

class QServoRenderNode : public QQuickItem
{
    Q_OBJECT

public:
    explicit QServoRenderNode(QQuickItem *parent = nullptr);

    QSGNode *updatePaintNode(QSGNode *node, UpdatePaintNodeData *) override;
};
