
#include "qservoglrendernode.h"

#include <QQuickItem>
#include <QDebug>
#include <QOpenGLFramebufferObject>
#include <QOpenGLContext>
#include <QOpenGLFunctions>

#include "cxx-qt-gen/servowebview.cxxqt.h"

#if QT_CONFIG(opengl)

QServoGLRenderNode::~QServoGLRenderNode()
{
    releaseResources();
}

void QServoGLRenderNode::releaseResources()
{

}

void QServoGLRenderNode::init()
{
    // TODO: create Servo now with GL context
}

void QServoGLRenderNode::render(const RenderState *state)
{
    Q_UNUSED(state);

    // TODO: trigger init() on first run

    if (m_swapChain.has_value())
    {
        qWarning() << Q_FUNC_INFO << "take surface as texture";
        auto texture = (*m_swapChain)->take_surface_as_texture();
        qWarning() << Q_FUNC_INFO << "got surface!";

        auto read_texture_id = texture->object();
        auto read_texture_target = texture->target();
        qWarning() << Q_FUNC_INFO << "read from: " << read_texture_id << read_texture_target;

        QOpenGLFunctions *f = QOpenGLContext::currentContext()->functions();

        QOpenGLFramebufferObject sourceFBO(400, 400);
        f->glFramebufferTexture2D(GL_READ_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, read_texture_target, read_texture_id, 0);
        qWarning() << Q_FUNC_INFO << "source valid: " << sourceFBO.isValid();

        QOpenGLFramebufferObject targetFBO(400, 400);
        targetFBO.bind();

        // QOpenGLFramebufferObject destFBO(400, 400);
        // f->glFramebufferTexture2D(GL_DRAW_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, 0, 0, 0);

        QOpenGLFramebufferObject::blitFramebuffer(&targetFBO, &sourceFBO);
        auto textureFBO = targetFBO.texture();
        qWarning() << Q_FUNC_INFO << "target: " << targetFBO.isValid() << targetFBO.size() << textureFBO;

        glBindTexture(GL_TEXTURE_2D, textureFBO);
        glBegin(GL_QUADS);
        glTexCoord2i(0, 0); glVertex2i(0, 0);
        glTexCoord2i(0, 1); glVertex2i(0, 400);
        glTexCoord2i(1, 1); glVertex2i(400, 400);
        glTexCoord2i(1, 0); glVertex2i(0, 0);
        glEnd();
        glBindTexture(GL_TEXTURE_2D, 0);
        glFlush();


        // auto image = targetFBO.toImage();
        // auto qsgTexture = window->createTextureFromImage(image);



        // TODO: render texture!

        // (*m_swapChain)->recycle_surface(std::move(surface));
    }

    // TODO: at this point do we render the WebRender context to the Qt context or something?

    // const auto* context = QOpenGLContext::currentContext();

    // TODO: here we need to tell webrender to render
}

QSGRenderNode::StateFlags QServoGLRenderNode::changedStates() const
{
    return BlendState | ScissorState | StencilState | DepthState;
}

QSGRenderNode::RenderingFlags QServoGLRenderNode::flags() const
{
    return BoundedRectRendering | DepthAwareRendering;
}

QRectF QServoGLRenderNode::rect() const
{
    return QRect(0, 0, m_width, m_height);
}

void QServoGLRenderNode::sync(QQuickItem *item)
{
    m_width = item->width();
    m_height = item->height();

    if (!m_swapChain.has_value())
    {
        if (auto webview = dynamic_cast<ServoWebView*>(item)) {
            // qWarning() << Q_FUNC_INFO << "found webview" << webview->loaded();
            if (webview->servoReady())
            {
                m_swapChain = std::move(webview->swapChain());
            }
            else
            {
                qWarning() << Q_FUNC_INFO << "waiting for servo";
            }
        } else {
            qWarning() << Q_FUNC_INFO << "missing webview";
        }
    }
}

#endif // opengl