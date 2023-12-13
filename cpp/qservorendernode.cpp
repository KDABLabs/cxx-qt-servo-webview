#include "qservorendernode.h"

#include <QSGRendererInterface>
#include <QQuickWindow>

#include "qservoglrendernode.h"

QServoRenderNode::QServoRenderNode(QQuickItem *parent)
    : QQuickItem(parent)
{
    setFlag(ItemHasContents);
}

QSGNode *QServoRenderNode::updatePaintNode(QSGNode *node, UpdatePaintNodeData *)
{
    QSGRenderNode *n = static_cast<QSGRenderNode *>(node);

    QSGRendererInterface *ri = window()->rendererInterface();
    if (!ri)
        return nullptr;

    switch (ri->graphicsApi()) {
    case QSGRendererInterface::OpenGL:
#if QT_CONFIG(opengl)
        if (!n)
            n = new QServoGLRenderNode;
        static_cast<QServoGLRenderNode *>(n)->sync(this);
#endif
        break;
    default:
        break;
    }

    if (!n)
        qWarning("QSGRendererInterface reports unknown graphics API %d", ri->graphicsApi());

    return n;
}
